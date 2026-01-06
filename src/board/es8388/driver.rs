use crate::board::es8388::command::Command;
use embedded_hal::digital::OutputPin;
use esp_idf_svc::hal::i2s::config::{
    DataBitWidth, MclkMultiple, SlotMode, StdClkConfig, StdConfig, StdSlotConfig,
};
use esp_idf_svc::hal::i2s::{config, I2sBiDir, I2sDriver};

pub const CHIP_ADDR: u8 = 0x10; // 芯片地址, ce是低电平时
#[allow(dead_code)]
pub struct Es8388<'d, I2C, EnSpk> {
    i2c: I2C,
    i2s: I2sDriver<'d, I2sBiDir>,
    en_spk: EnSpk,
    addr: u8,
}

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
    ) -> anyhow::Result<Self> {
        let mut es8388 = Es8388 {
            i2c,
            i2s,
            en_spk,
            addr,
        };
        es8388.test_i2c_rw()?;
        Ok(es8388)
    }

    /// 写入寄存器
    pub fn write_reg(&mut self, reg: Command, val: u8) -> anyhow::Result<()> {
        self.i2c
            .write(self.addr, &[reg.reg_addr(), val])
            .map_err(|e| anyhow::anyhow!("I2C Write Error: {:?}", e))
    }

    /// 读取寄存器
    pub fn read_reg(&mut self, reg: Command) -> anyhow::Result<u8> {
        let mut buffer = [0u8; 1];
        self.i2c
            .write_read(self.addr, &[reg.reg_addr()], &mut buffer)
            .map_err(|e| anyhow::anyhow!("I2C Read Error: {:?}", e))?;
        Ok(buffer[0])
    }

    /// 播放音频 (写入 I2S)
    /// buffer: 16-bit PCM 数据
    /// timeout_ms: 写入超时时间
    pub fn write_audio(&mut self, data: &[u8], timeout_ms: u32) -> anyhow::Result<usize> {
        // I2sDriver 已经在 new_std_bidir 中初始化为双向
        self.i2s
            .write(data, timeout_ms)
            .map_err(|e| anyhow::anyhow!("I2S Write Error: {:?}", e))
    }

    /// 录制音频 (读取 I2S)
    /// buffer: 存放读取到的 PCM 数据
    pub fn read_audio(&mut self, buffer: &mut [u8], timeout_ms: u32) -> anyhow::Result<usize> {
        self.i2s
            .read(buffer, timeout_ms)
            .map_err(|e| anyhow::anyhow!("I2S Read Error: {:?}", e))
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
pub fn default_i2s_config() -> StdConfig {
    let channel_cfg = config::Config::default();
    let i2s_std_clk_config = StdClkConfig::new(44100, Default::default(), MclkMultiple::M256);
    let i2s_slot_cfg = StdSlotConfig::philips_slot_default(DataBitWidth::Bits16, SlotMode::Mono);
    StdConfig::new(
        channel_cfg,
        i2s_std_clk_config,
        i2s_slot_cfg,
        Default::default(),
    )
}
