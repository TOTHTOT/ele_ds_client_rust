//! ES8388 音频 CODEC 芯片 Rust 命令枚举
//! 基于 ES8388 官方 datasheet (Revision 5.0) 精准定义
//! 涵盖所有核心控制寄存器、位配置及功能子枚举

//! ES8388 芯片核心控制命令（寄存器地址映射）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    // -------------------------- 芯片控制与电源管理 (6.1 节) --------------------------
    /// 寄存器 0: 芯片控制 1 (默认值 0x06 = 00000110)
    ChipControl1 = 0x00,
    /// 寄存器 1: 芯片控制 2 (默认值 0x5C = 01011100)
    ChipControl2 = 0x01,
    /// 寄存器 2: 芯片电源管理 (默认值 0xC3 = 11000011)
    ChipPowerManagement = 0x02,
    /// 寄存器 3: ADC 电源管理 (默认值 0xFC = 11111100)
    AdcPowerManagement = 0x03,
    /// 寄存器 4: DAC 电源管理 (默认值 0xC0 = 11000000)
    DacPowerManagement = 0x04,
    /// 寄存器 5: 芯片低功耗 1 (默认值 0x00)
    ChipLowPower1 = 0x05,
    /// 寄存器 6: 芯片低功耗 2 (默认值 0x00)
    ChipLowPower2 = 0x06,
    /// 寄存器 7: 模拟电压管理 (默认值 0x7C = 01111100)
    AnalogVoltageManagement = 0x07,
    /// 寄存器 8: 主模式控制, 串行端口模式、主时钟分频、位时钟极性及分频系数 (默认值 0x80 = 10000000)
    MasterModeControl = 0x08,

    // -------------------------- ADC 控制 (6.2 节) --------------------------
    /// 寄存器 9: ADC 控制 1 (麦克风增益) (默认值 0x00)
    AdcControl1 = 0x09,
    /// 寄存器 10: ADC 控制 2 (输入通道选择) (默认值 0x00)
    AdcControl2 = 0x0A,
    /// 寄存器 11: ADC 控制 3 (差分输入/单声道混合) (默认值 0x02 = 00000010)
    AdcControl3 = 0x0B,
    /// 寄存器 12: ADC 控制 4 (数据格式/位宽) (默认值 0x00)
    AdcControl4 = 0x0C,
    /// 寄存器 13: ADC 控制 5 (采样率模式/比例) (默认值 0x06 = 00000110)
    AdcControl5 = 0x0D,
    /// 寄存器 14: ADC 控制 6 (极性反转/高通滤波) (默认值 0x30 = 00110000)
    AdcControl6 = 0x0E,
    /// 寄存器 15: ADC 控制 7 (音量斜坡/软斜坡) (默认值 0x20 = 00100000)
    AdcControl7 = 0x0F,
    /// 寄存器 16: ADC 控制 8 (左声道数字音量) (默认值 0xC0 = 11000000)
    AdcControl8 = 0x10,
    /// 寄存器 17: ADC 控制 9 (右声道数字音量) (默认值 0xC0 = 11000000)
    AdcControl9 = 0x11,
    /// 寄存器 18: ADC 控制 10 (ALC 配置) (默认值 0x38 = 00111000)
    AdcControl10 = 0x12,
    /// 寄存器 19: ADC 控制 11 (ALC 目标/保持时间) (默认值 0xB0 = 10110000)
    AdcControl11 = 0x13,
    /// 寄存器 20: ADC 控制 12 (ALC 衰减/攻击时间) (默认值 0x32 = 00110010)
    AdcControl12 = 0x14,
    /// 寄存器 21: ADC 控制 13 (ALC 模式/零交叉) (默认值 0x06 = 00000110)
    AdcControl13 = 0x15,
    /// 寄存器 22: ADC 控制 14 (噪声门阈值) (默认值 0x00)
    AdcControl14 = 0x16,

    // -------------------------- DAC 控制 (6.3 节) --------------------------
    /// 寄存器 23: DAC 控制 1 (声道交换/数据格式) (默认值 0x00)
    DacControl1 = 0x17,
    /// 寄存器 24: DAC 控制 2 (采样率模式/比例) (默认值 0x06 = 00000110)
    DacControl2 = 0x18,
    /// 寄存器 25: DAC 控制 3 (音量斜坡/软斜坡) (默认值 0x22 = 00100010)
    DacControl3 = 0x19,
    /// 寄存器 26: DAC 控制 4 (左声道数字音量) (默认值 0xC0 = 11000000)
    DacControl4 = 0x1A,
    /// 寄存器 27: DAC 控制 5 (右声道数字音量) (默认值 0xC0 = 11000000)
    DacControl5 = 0x1B,
    /// 寄存器 28: DAC 控制 6 (去加重/极性反转) (默认值 0x08 = 00001000)
    DacControl6 = 0x1C,
    /// 寄存器 29: DAC 控制 7 (单声道/输出幅度) (默认值 0x00)
    DacControl7 = 0x1D,
    /// 寄存器 30: DAC 控制 8 (搁架滤波器系数 a[29:24]) (默认值 0x1F = 00011111)
    DacControl8 = 0x1E,
    /// 寄存器 31: DAC 控制 9 (搁架滤波器系数 a[23:16]) (默认值 0xF7 = 11110111)
    DacControl9 = 0x1F,
    /// 寄存器 32: DAC 控制 10 (搁架滤波器系数 a[15:8]) (默认值 0xFD = 11111101)
    DacControl10 = 0x20,
    /// 寄存器 33: DAC 控制 11 (搁架滤波器系数 a[7:0]) (默认值 0xFF = 11111111)
    DacControl11 = 0x21,
    /// 寄存器 34: DAC 控制 12 (搁架滤波器系数 b[29:24]) (默认值 0x1F = 00011111)
    DacControl12 = 0x22,
    /// 寄存器 35: DAC 控制 13 (搁架滤波器系数 b[23:16]) (默认值 0xF7 = 11110111)
    DacControl13 = 0x23,
    /// 寄存器 36: DAC 控制 14 (搁架滤波器系数 b[15:8]) (默认值 0xFD = 11111101)
    DacControl14 = 0x24,
    /// 寄存器 37: DAC 控制 15 (搁架滤波器系数 b[7:0]) (默认值 0xFF = 11111111)
    DacControl15 = 0x25,
    /// 寄存器 38: DAC 控制 16 (输出混合输入选择) (默认值 0x00)
    DacControl16 = 0x26,
    /// 寄存器 39: DAC 控制 17 (左声道混合增益) (默认值 0x38 = 00111000)
    DacControl17 = 0x27,
    /// 寄存器 40: DAC 控制 18 (保留) (默认值 0x28 = 00101000)
    DacControl18 = 0x28,
    /// 寄存器 41: DAC 控制 19 (保留) (默认值 0x28 = 00101000)
    DacControl19 = 0x29,
    /// 寄存器 42: DAC 控制 20 (右声道混合增益) (默认值 0x38 = 00111000)
    DacControl20 = 0x2A,
    /// 寄存器 43: DAC 控制 21 (时钟/DLL 控制) (默认值 0x00)
    DacControl21 = 0x2B,
    /// 寄存器 44: DAC 控制 22 (DC 偏移) (默认值 0x00)
    DacControl22 = 0x2C,
    /// 寄存器 45: DAC 控制 23 (VREF 输出电阻) (默认值 0x00)
    DacControl23 = 0x2D,
    /// 寄存器 46: DAC 控制 24 (LOUT1 音量) (默认值 0x00)
    DacControl24 = 0x2E,
    /// 寄存器 47: DAC 控制 25 (ROUT1 音量) (默认值 0x00)
    DacControl25 = 0x2F,
    /// 寄存器 48: DAC 控制 26 (LOUT2 音量) (默认值 0x00)
    DacControl26 = 0x30,
    /// 寄存器 49: DAC 控制 27 (ROUT2 音量) (默认值 0x00)
    DacControl27 = 0x31,
    /// 寄存器 50: DAC 控制 28 (保留) (默认值 0x00)
    DacControl28 = 0x32,
    /// 寄存器 51: DAC 控制 29 (输出参考配置) (默认值 0xAA = 10101010)
    DacControl29 = 0x33,
    /// 寄存器 52: DAC 控制 30 (混音参考配置) (默认值 0xAA = 10101010)
    DacControl30 = 0x34,
}

impl Command {
    /// 获取寄存器地址
    pub fn reg_addr(&self) -> u8 {
        *self as u8
    }

    /// 获取寄存器默认值（基于 datasheet 定义）
    pub fn default_value(&self) -> u8 {
        match self {
            Self::ChipControl1 => 0x06,
            Self::ChipControl2 => 0x5C,
            Self::ChipPowerManagement => 0xC3,
            Self::AdcPowerManagement => 0xFC,
            Self::DacPowerManagement => 0xC0,
            Self::AnalogVoltageManagement => 0x7C,
            Self::MasterModeControl => 0x80,
            Self::AdcControl3 => 0x02,
            Self::AdcControl5 => 0x06,
            Self::AdcControl6 => 0x30,
            Self::AdcControl7 => 0x20,
            Self::AdcControl8 => 0xC0,
            Self::AdcControl9 => 0xC0,
            Self::AdcControl10 => 0x38,
            Self::AdcControl11 => 0xB0,
            Self::AdcControl12 => 0x32,
            Self::AdcControl13 => 0x06,
            Self::DacControl2 => 0x06,
            Self::DacControl3 => 0x22,
            Self::DacControl4 => 0xC0,
            Self::DacControl5 => 0xC0,
            Self::DacControl6 => 0x08,
            Self::DacControl8 => 0x1F,
            Self::DacControl9 => 0xF7,
            Self::DacControl10 => 0xFD,
            Self::DacControl11 => 0xFF,
            Self::DacControl12 => 0x1F,
            Self::DacControl13 => 0xF7,
            Self::DacControl14 => 0xFD,
            Self::DacControl15 => 0xFF,
            Self::DacControl17 => 0x38,
            Self::DacControl18 => 0x28,
            Self::DacControl19 => 0x28,
            Self::DacControl20 => 0x38,
            Self::DacControl29 => 0xAA,
            Self::DacControl30 => 0xAA,
            _ => 0x00, // 其余寄存器默认值为 0x00
        }
    }

    /// 从寄存器地址解析命令
    pub fn from_reg_addr(addr: u8) -> Option<Self> {
        match addr {
            0x00 => Some(Self::ChipControl1),
            0x01 => Some(Self::ChipControl2),
            0x02 => Some(Self::ChipPowerManagement),
            0x03 => Some(Self::AdcPowerManagement),
            0x04 => Some(Self::DacPowerManagement),
            0x05 => Some(Self::ChipLowPower1),
            0x06 => Some(Self::ChipLowPower2),
            0x07 => Some(Self::AnalogVoltageManagement),
            0x08 => Some(Self::MasterModeControl),
            0x09 => Some(Self::AdcControl1),
            0x0A => Some(Self::AdcControl2),
            0x0B => Some(Self::AdcControl3),
            0x0C => Some(Self::AdcControl4),
            0x0D => Some(Self::AdcControl5),
            0x0E => Some(Self::AdcControl6),
            0x0F => Some(Self::AdcControl7),
            0x10 => Some(Self::AdcControl8),
            0x11 => Some(Self::AdcControl9),
            0x12 => Some(Self::AdcControl10),
            0x13 => Some(Self::AdcControl11),
            0x14 => Some(Self::AdcControl12),
            0x15 => Some(Self::AdcControl13),
            0x16 => Some(Self::AdcControl14),
            0x17 => Some(Self::DacControl1),
            0x18 => Some(Self::DacControl2),
            0x19 => Some(Self::DacControl3),
            0x1A => Some(Self::DacControl4),
            0x1B => Some(Self::DacControl5),
            0x1C => Some(Self::DacControl6),
            0x1D => Some(Self::DacControl7),
            0x1E => Some(Self::DacControl8),
            0x1F => Some(Self::DacControl9),
            0x20 => Some(Self::DacControl10),
            0x21 => Some(Self::DacControl11),
            0x22 => Some(Self::DacControl12),
            0x23 => Some(Self::DacControl13),
            0x24 => Some(Self::DacControl14),
            0x25 => Some(Self::DacControl15),
            0x26 => Some(Self::DacControl16),
            0x27 => Some(Self::DacControl17),
            0x28 => Some(Self::DacControl18),
            0x29 => Some(Self::DacControl19),
            0x2A => Some(Self::DacControl20),
            0x2B => Some(Self::DacControl21),
            0x2C => Some(Self::DacControl22),
            0x2D => Some(Self::DacControl23),
            0x2E => Some(Self::DacControl24),
            0x2F => Some(Self::DacControl25),
            0x30 => Some(Self::DacControl26),
            0x31 => Some(Self::DacControl27),
            0x32 => Some(Self::DacControl28),
            0x33 => Some(Self::DacControl29),
            0x34 => Some(Self::DacControl30),
            _ => None,
        }
    }
}

// -------------------------- 配套子枚举（寄存器位配置定义） --------------------------

/// 寄存器 0 (ChipControl1) - VMID 分压电阻选择
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VmidDivider {
    Disabled = 0x00,    // 00 - VMID 禁用
    Divider50k = 0x01,  // 01 - 50kΩ 分压（默认）
    Divider500k = 0x02, // 10 - 500kΩ 分压
    Divider5k = 0x03,   // 11 - 5kΩ 分压
}

/// 寄存器 8 (MasterModeControl) - 串口模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SerialPortMode {
    Slave = 0x00,  // 0 - 从模式
    Master = 0x80, // 1 - 主模式（默认）
}

/// 寄存器 9/10 (AdcControl1) - 麦克风 PGA 增益
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MicPgaGain {
    Gain0Db = 0x00,  // 0000 - 0dB（默认）
    Gain3Db = 0x01,  // 0001 - +3dB
    Gain6Db = 0x02,  // 0010 - +6dB
    Gain9Db = 0x03,  // 0011 - +9dB
    Gain12Db = 0x04, // 0100 - +12dB
    Gain15Db = 0x05, // 0101 - +15dB
    Gain18Db = 0x06, // 0110 - +18dB
    Gain21Db = 0x07, // 0111 - +21dB
    Gain24Db = 0x08, // 1000 - +24dB
}

/// 寄存器 10 (AdcControl2) - ADC 输入通道选择
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AdcInputChannel {
    Input1 = 0x00,       // 00 - 输入通道 1（默认）
    Input2 = 0x01,       // 01 - 输入通道 2
    Reserved = 0x02,     // 10 - 保留
    Differential = 0x03, // 11 - 差分输入（LIN-RIN）
}

/// 寄存器 12/23 (AdcControl4/DacControl1) - 音频数据格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AudioDataFormat {
    I2s = 0x00,            // 00 - I2S 格式（默认）
    LeftJustified = 0x01,  // 01 - 左对齐格式
    RightJustified = 0x02, // 10 - 右对齐格式
    DspPcm = 0x03,         // 11 - DSP/PCM 模式
}

/// 寄存器 12/23 (AdcControl4/DacControl1) - 音频数据位宽
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AudioBitWidth {
    Bit24 = 0x00, // 000 - 24位（默认）
    Bit20 = 0x01, // 001 - 20位
    Bit18 = 0x02, // 010 - 18位
    Bit16 = 0x03, // 011 - 16位
    Bit32 = 0x04, // 100 - 32位
}

/// 寄存器 13/24 (AdcControl5/DacControl2) - 采样率模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SampleRateMode {
    SingleSpeed = 0x00, // 0 - 单速模式（8kHz-50kHz，默认）
    DoubleSpeed = 0x20, // 1 - 双速模式（50kHz-100kHz）
}

/// 寄存器 13/24 (AdcControl5/DacControl2) - 主模式 MCLK/LRCK 比例（常用值）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MclkLrckRatio {
    Ratio128 = 0x00,  // 00000 - 128（双速模式）
    Ratio256 = 0x02,  // 00010 - 256（默认单速模式）
    Ratio384 = 0x03,  // 00011 - 384
    Ratio512 = 0x04,  // 00100 - 512
    Ratio768 = 0x06,  // 00110 - 768（默认）
    Ratio1024 = 0x07, // 00111 - 1024
}

/// 寄存器 18 (AdcControl10) - ALC 工作模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlcMode {
    Off = 0x00,       // 00 - ALC 关闭（默认）
    RightOnly = 0x40, // 01 - 仅右声道 ALC
    LeftOnly = 0x80,  // 10 - 仅左声道 ALC
    Stereo = 0xC0,    // 11 - 立体声 ALC
}

/// 寄存器 28 (DacControl6) - 去加重模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeemphasisMode {
    Disabled = 0x00,  // 00 - 禁用（默认）
    Fs32kHz = 0x40,   // 01 - 32kHz 去加重（单速模式）
    Fs44_1kHz = 0x80, // 10 - 44.1kHz 去加重（单速模式）
    Fs48kHz = 0xC0,   // 11 - 48kHz 去加重（单速模式）
}

/// 寄存器 29 (DacControl7) - DAC 输出模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DacOutputMode {
    Stereo = 0x00, // 0 - 立体声（默认）
    Mono = 0x20,   // 1 - 单声道（L+R)/2
}

/// 寄存器 29 (DacControl7) - DAC 输出幅度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DacOutputVpp {
    V3_5 = 0x00, // 00 - 3.5Vpp（0.7调制指数，默认）
    V4_0 = 0x01, // 01 - 4.0Vpp
    V3_0 = 0x02, // 10 - 3.0Vpp
    V2_5 = 0x03, // 11 - 2.5Vpp
}

// -------------------------- 寄存器位操作辅助方法 --------------------------
impl Command {
    /// 构建 ADC 输入通道配置数据（寄存器 10）
    pub fn build_adc_input_config(left_chan: AdcInputChannel, right_chan: AdcInputChannel) -> u8 {
        ((left_chan as u8) << 6) | ((right_chan as u8) << 4)
    }

    /// 构建音频格式配置数据（寄存器 12/23）
    pub fn build_audio_format_config(format: AudioDataFormat, bit_width: AudioBitWidth) -> u8 {
        ((bit_width as u8) << 2) | (format as u8)
    }

    /// 构建采样率配置数据（寄存器 13/24）
    pub fn build_sample_rate_config(mode: SampleRateMode, ratio: MclkLrckRatio) -> u8 {
        (mode as u8) | (ratio as u8)
    }

    /// 构建 DAC 输出配置数据（寄存器 29）
    pub fn build_dac_output_config(mode: DacOutputMode, vpp: DacOutputVpp) -> u8 {
        (mode as u8) | (vpp as u8)
    }
}
