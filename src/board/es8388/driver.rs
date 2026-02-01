use crate::board::es8388::command::Command;
use esp_idf_svc::hal::i2s::config::{
    ClockSource, DataBitWidth, MclkMultiple, SlotMode, StdClkConfig, StdConfig, StdSlotConfig,
};
use std::cmp::PartialEq;

pub const CHIP_ADDR: u8 = 0x10; // 芯片地址, ce是低电平时
#[allow(dead_code)]
pub struct Es8388<I2C> {
    i2c: I2C,
    addr: u8,
    mode: RunMode,
}

#[derive(PartialEq, Debug)]
#[allow(dead_code)]
pub enum RunMode {
    Adc,
    Dac,
    AdcDac,
}

#[allow(dead_code)]
impl<I2C> Es8388<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    pub fn new(i2c: I2C, addr: u8, mode: RunMode) -> Self {
        Es8388 { i2c, addr, mode }
    }

    /// 初始化芯片, 修复寄存器配置逻辑
    pub fn init(&mut self) -> anyhow::Result<()> {
        // 1. 软复位 ES8388
        self.write_reg(Command::ChipControl1, 0x80)?;
        self.write_reg(Command::ChipControl1, 0x00)?;
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 2. 芯片控制与电源管理启动序列
        self.write_reg(Command::ChipControl2, 0x58)?;
        self.write_reg(Command::ChipControl2, 0x50)?;
        self.write_reg(Command::ChipPowerManagement, 0xF3)?;
        self.write_reg(Command::ChipPowerManagement, 0xF0)?;

        // 3. 电源管理与时钟配置
        self.write_reg(Command::AdcPowerManagement, 0x09)?; // 麦克风偏置关闭
        self.write_reg(Command::ChipControl1, 0x06)?; // 参考/500K驱动使能
        self.write_reg(Command::DacPowerManagement, 0x00)?; // DAC通道暂不打开
        self.write_reg(Command::MasterModeControl, 0x00)?; // MCLK不分频
        self.write_reg(Command::DacControl21, 0x80)?; // DACLRC 与 ADCLRC 相同

        // 4. ADC 配置 (录音)
        self.write_reg(Command::AdcControl1, 0x88)?; // PGA增益 +24dB
        self.write_reg(Command::AdcControl4, 0x4C)?; // 16bit, Left ADC data
        self.write_reg(Command::AdcControl5, 0x02)?; // MCLK/LRCK = 256
        self.write_reg(Command::AdcControl8, 0x00)?; // 左声道音量衰减最小
        self.write_reg(Command::AdcControl9, 0x00)?; // 右声道音量衰减最小

        // 5. DAC 配置 (播放)
        self.write_reg(Command::DacControl1, 0x18)?; // 16bit 格式
        self.write_reg(Command::DacControl2, 0x02)?; // MCLK/LRCK = 256
        self.write_reg(Command::DacControl4, 0xc0)?; // 左数字音量
        self.write_reg(Command::DacControl5, 0xc0)?; // 右数字音量
        self.write_reg(Command::DacControl17, 0xB8)?; // L混频器配置
        self.write_reg(Command::DacControl20, 0xB8)?; // R混频器配置
        std::thread::sleep(std::time::Duration::from_millis(100));
        Ok(())
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.set_adda_cfg(true, false)?;
        self.write_reg(Command::DacControl4, 0x00)?; // 左数字音量
        self.write_reg(Command::DacControl5, 0x00)?; // 右数字音量
        self.set_input_cfg(0)?;
        self.set_output_cfg(true, true)?;
        self.set_spk_volume(25)
    }

    /// 写入寄存器
    pub fn write_reg(&mut self, reg: Command, val: u8) -> anyhow::Result<()> {
        self.i2c
            .write(self.addr, &[reg.reg_addr(), val])
            .map_err(|e| anyhow::anyhow!("I2C Write Error: {e:?}"))
    }

    /// 读取寄存器
    pub fn read_reg(&mut self, reg: Command) -> anyhow::Result<u8> {
        let mut buffer = [0u8; 1];
        self.i2c
            .write_read(self.addr, &[reg.reg_addr()], &mut buffer)
            .map_err(|e| anyhow::anyhow!("I2C Read Error: {e:?}"))?;
        Ok(buffer[0])
    }

    /// 读取所有寄存器（调试用）
    pub fn read_all(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();
        for reg in 0..50 {
            let Some(cmd) = Command::from_reg_addr(reg) else {
                buf.push(0);
                continue;
            };
            let tmp = self.read_reg(cmd)?;
            buf.push(tmp);
        }
        Ok(buf)
    }

    pub fn set_adda_cfg(&mut self, dac_en: bool, adc_en: bool) -> anyhow::Result<()> {
        let mut val = 0u8;
        if !dac_en {
            val |= (1 << 0) | (1 << 2);
        }
        if !adc_en {
            val |= (1 << 1) | (1 << 3);
        }
        self.write_reg(Command::ChipPowerManagement, val)
    }

    /// 对应 es8388_output_cfg: DAC 输出通道配置
    /// o1en: LOUT1/ROUT1 使能, o2en: LOUT2/ROUT2 使能
    pub fn set_output_cfg(&mut self, o1en: bool, o2en: bool) -> anyhow::Result<()> {
        let mut val = 0u8;
        if o1en {
            val |= 3 << 4;
        } // 通道 1 (110000)
        if o2en {
            val |= 3 << 2;
        } // 通道 2 (001100)
        self.write_reg(Command::DacPowerManagement, val)
    }

    /// 对应 es8388_sai_cfg: 设置工作模式和数据长度
    /// fmt: 0-飞利浦I2S, 1-MSB, 2-LSB, 3-PCM/DSP
    /// len: 0-24bit, 1-20bit, 2-18bit, 3-16bit, 4-32bit
    pub fn set_sai_cfg(&mut self, fmt: u8, len: u8) -> anyhow::Result<()> {
        let val = ((fmt & 0x03) << 1) | ((len & 0x07) << 3);
        self.write_reg(Command::DacControl23, val) // R23
    }

    /// 设置耳机音量 (Reg 0x2E/0x2F)
    pub fn set_hp_volume(&mut self, volume: u8) -> anyhow::Result<()> {
        let vol = volume.min(33);
        self.write_reg(Command::DacControl24, vol)?;
        self.write_reg(Command::DacControl25, vol)
    }

    /// 设置喇叭音量 (Reg 0x30/0x31)
    pub fn set_spk_volume(&mut self, volume: u8) -> anyhow::Result<()> {
        let vol = volume.min(33);
        self.write_reg(Command::DacControl26, vol)?;
        self.write_reg(Command::DacControl27, vol)?;
        self.write_reg(Command::DacControl24, vol)?;
        self.write_reg(Command::DacControl25, vol)
    }

    /// 对应 es8388_mic_gain: 设置 MIC PGA 增益 (0~8, 对应 0~24dB)
    pub fn set_mic_gain(&mut self, gain: u8) -> anyhow::Result<()> {
        let g = (gain & 0x0F) | ((gain & 0x0F) << 4);
        self.write_reg(Command::AdcControl1, g) // R9
    }

    /// 对应 es8388_input_cfg: ADC 输入通道配置
    /// in_sel: 0-通道1, 1-通道2
    pub fn set_input_cfg(&mut self, in_sel: u8) -> anyhow::Result<()> {
        let val = (5 * (in_sel & 0x01)) << 4;
        self.write_reg(Command::AdcControl2, val) // R10
    }

    /// 对应 es8388_3d_set: 3D 环绕声设置 (0-关闭, 7-最强)
    pub fn set_3d(&mut self, depth: u8) -> anyhow::Result<()> {
        self.write_reg(Command::DacControl7, (depth & 0x07) << 2) // R29/0x1D
    }

    /// 对应 es8388_alc_ctrl: ALC 自动电平控制设置
    pub fn set_alc_ctrl(&mut self, sel: u8, max_gain: u8, min_gain: u8) -> anyhow::Result<()> {
        let mut val = (sel & 0x03) << 6;
        val |= (max_gain & 0x07) << 3;
        val |= min_gain & 0x07;
        self.write_reg(Command::AdcControl10, val) // R18/0x12
    }
}

/// 修复I2S配置：确保时序匹配
#[allow(dead_code)]
pub fn default_i2s_config() -> StdConfig {
    let channel_cfg = esp_idf_svc::hal::i2s::config::Config::default();
    let i2s_std_clk_config = StdClkConfig::new(44100, ClockSource::Pll160M, MclkMultiple::M256);
    let i2s_slot_cfg = StdSlotConfig::philips_slot_default(DataBitWidth::Bits16, SlotMode::Mono);
    StdConfig::new(
        channel_cfg,
        i2s_std_clk_config,
        i2s_slot_cfg,
        Default::default(),
    )
}
