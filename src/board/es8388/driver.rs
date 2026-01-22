use crate::board::es8388::command::Command;
use embedded_hal::digital::OutputPin;
use esp_idf_svc::hal::i2s::config::{
    DataBitWidth, MclkMultiple, SlotMode, StdClkConfig, StdConfig, StdSlotConfig,
};
use esp_idf_svc::hal::i2s::{config, I2sBiDir, I2sDriver};
use std::cmp::PartialEq;

pub const CHIP_ADDR: u8 = 0x10; // 芯片地址, ce是低电平时
#[allow(dead_code)]
pub struct Es8388<'d, I2C, EnSpk> {
    i2c: I2C,
    i2s: I2sDriver<'d, I2sBiDir>,
    en_spk: EnSpk,
    addr: u8,
    mode: RunMode,
}

#[derive(PartialEq)]
pub enum RunMode {
    Adc,
    Dac,
    AdcDac,
}

#[allow(dead_code)]
impl<'d, I2C, EnSpk> Es8388<'d, I2C, EnSpk>
where
    I2C: embedded_hal::i2c::I2c,
    EnSpk: OutputPin,
{
    pub fn new(
        i2s: I2sDriver<'d, I2sBiDir>,
        i2c: I2C,
        en_spk: EnSpk,
        addr: u8,
        mode: RunMode,
    ) -> Self {
        Es8388 {
            i2c,
            i2s,
            en_spk,
            addr,
            mode,
        }
    }

    /// 初始化芯片, 一些寄存器会先变成复位状态, 知道真正开始时才会设置值, 在start()函数中配置
    pub fn init(&mut self) -> anyhow::Result<()> {
        // 使用默认值的可以不发送
        self.write_reg(Command::ChipControl1, 0b0001_0110)?;
        self.write_reg(Command::ChipControl2, Command::ChipControl2.default_value())?;
        // 电源相关的全部开启, 测试完成后可以考虑关闭部分不使用的功能
        self.write_reg(Command::ChipPowerManagement, 0x00)?;
        self.write_reg(Command::MasterModeControl, 0x00)?;

        if self.mode == RunMode::Dac || self.mode == RunMode::AdcDac {
            self.write_reg(Command::AdcPowerManagement, 0xff)?; // 先复位然后在start中配置
            self.write_reg(Command::DacPowerManagement, 0x00)?;

            self.write_reg(Command::DacControl1, 0b0001_1000)?;
            self.write_reg(Command::DacControl2, 0b0000_0010)?; // 设置采样频率, 要和i2s配置的一样, 具体值查表
            self.write_reg(Command::DacControl16, 0x00)?;
            self.write_reg(Command::DacControl17, 0x90)?; // 左咪头声音不混入扬声器
            self.write_reg(Command::DacControl20, 0x90)?; // 右咪头声音不混入扬声器
            self.write_reg(Command::DacControl21, 0x80)?;
            self.write_reg(Command::DacControl23, 0x00)?; // 扬声器规格是8欧5w的
            self.set_dac_volume(100)?; // 设置输入信号的增幅, 这里直接最大
        }

        if self.mode == RunMode::Adc || self.mode == RunMode::AdcDac {
            // 配置adc相关寄存器
            // self.write_reg(Command::AdcPowerManagement, 0x80)?;
            self.write_reg(Command::AdcControl1, 0x22)?; // 看别的代码配的是0xbb, 看手册没这个匹配的模式, 这里改为我自己认为正确的
            self.write_reg(Command::AdcControl2, 0x00)?;
            self.write_reg(Command::AdcControl3, 0b0000_0000)?;
            self.write_reg(Command::AdcControl4, 0b0000_1100)?;
            self.write_reg(Command::AdcControl5, 0x02)?;
        }
        Ok(())
    }

    /// 配置完成后在启动, 避免不需要芯片工作时带来的功耗
    pub fn start(&mut self) -> anyhow::Result<()> {
        log::info!("Starting i2s");
        self.i2s.rx_enable()?;
        self.i2s.tx_enable()?;
        if self.mode == RunMode::Adc || self.mode == RunMode::AdcDac {
            self.write_reg(Command::AdcPowerManagement, 0x00)?;
        }
        if self.mode == RunMode::Dac || self.mode == RunMode::AdcDac {
            self.write_reg(Command::DacPowerManagement, 0b1111_1100)?;
            self.write_reg(Command::DacControl17, 0x50)?;
            self.write_reg(Command::DacControl20, 0x50)?;
            self.set_voice_volume(50)?;
        }
        Ok(())
    }

    fn get_adc_dac_volume_from_arg(volume: u8) -> u8 {
        let pct = volume.min(100);
        let target_db = -((100 - pct) as f32 * 96.0 / 100.0);
        let reg_val = (target_db.abs() / 0.5) as u8;
        reg_val.min(192)
    }
    /// 设置音量输出, 左右声道音量相同, 这里调节的是输入的数字型号
    pub fn set_dac_volume(&mut self, volume: u8) -> anyhow::Result<()> {
        let volume = Self::get_adc_dac_volume_from_arg(volume);
        self.write_reg(Command::DacControl4, volume)?;
        self.write_reg(Command::DacControl5, volume)?;
        Ok(())
    }

    /// 将音量转为芯片音量的对应bit
    fn get_voice_volume_from_arg(volume: u8) -> u8 {
        let pct = volume;
        let target_db = -((100 - pct) as f32 * 96.0 / 100.0);
        let reg_val = (target_db.abs() / 1.5) as u8;
        reg_val.min(30)
        // volume.max(100) / 3
    }
    /// 设置音量输出, 左右声道音量相同, 这里调节的是最终输出的模拟信号
    pub fn set_voice_volume(&mut self, volume: u8) -> anyhow::Result<()> {
        let volume = Self::get_voice_volume_from_arg(volume);
        self.write_reg(Command::DacControl24, volume)?;
        self.write_reg(Command::DacControl25, volume)?;
        Ok(())
    }

    pub fn read_all(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();
        for reg in 0..50 {
            let Some(cmd) = Command::from_reg_addr(reg) else {
                continue;
            };
            let tmp = self.read_reg(cmd)?;
            buf.push(tmp);
        }
        Ok(buf)
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

    /// 播放音频 (写入 I2S)
    /// buffer: 16-bit PCM 数据
    /// timeout_ms: 写入超时时间
    pub fn write_audio(&mut self, data: &[u8], timeout_ms: u32) -> anyhow::Result<usize> {
        // self.set_speaker(true)?;
        let size = self
            .i2s
            .write(data, timeout_ms)
            .map_err(|e| anyhow::anyhow!("I2S Write Error: {e:?}"))?;
        // self.set_speaker(false)?;
        Ok(size)
    }

    /// 录制音频 (读取 I2S)
    /// buffer: 存放读取到的 PCM 数据
    pub fn read_audio(&mut self, buffer: &mut [u8], timeout_ms: u32) -> anyhow::Result<usize> {
        self.i2s
            .read(buffer, timeout_ms)
            .map_err(|e| anyhow::anyhow!("I2S Read Error: {e:?}"))
    }

    /// 开启扬声器
    pub fn set_speaker(&mut self, on: bool) -> anyhow::Result<()> {
        if on {
            self.en_spk
                .set_high()
                .map_err(|_| anyhow::anyhow!("GPIO Error"))?;
        } else {
            self.en_spk
                .set_low()
                .map_err(|_| anyhow::anyhow!("GPIO Error"))?;
        }
        Ok(())
    }

    /// 测试i2c读写, 通过读写寄存器实现
    fn test_i2c_rw(&mut self) -> anyhow::Result<()> {
        let write_val = 0xff;
        self.write_reg(Command::ChipControl1, write_val)?;
        let read_val = self.read_reg(Command::ChipControl1)?;
        log::info!("write is: {write_val}, read is: {read_val}");
        Ok(())
    }
}

// 生成默认的i2s配置
pub fn default_i2s_config() -> StdConfig {
    let channel_cfg = config::Config::default();
    let i2s_std_clk_config = StdClkConfig::new(44100, Default::default(), MclkMultiple::M256);
    let i2s_slot_cfg = StdSlotConfig::philips_slot_default(DataBitWidth::Bits16, SlotMode::Stereo);
    StdConfig::new(
        channel_cfg,
        i2s_std_clk_config,
        i2s_slot_cfg,
        Default::default(),
    )
}
