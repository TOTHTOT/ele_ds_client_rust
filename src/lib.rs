// src/lib.rs

use serde::{Deserialize, Serialize};

pub mod board;
pub mod cmd_menu;
pub mod communication;
pub mod device_config;
pub mod file_system;
pub mod ui;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(usize)]
pub enum ActivePage {
    Sensor, // 单击左边按键
    #[default]
    Home, // 单击中间按键
    Image,  // 单击右边按键

    FullTIme,    // 双击左边按键
    Setting,     // 双击中间按键
    FullWeather, // 双击右边按键

    About, // 三击中间按键
    None,
}

impl ActivePage {
    /// 当前页面是否需要在每次收到更新命令时刷新
    pub fn cur_set_page_is_need_refresh(self) -> bool {
        if self == ActivePage::Home || self == ActivePage::Sensor {
            return true;
        }
        false
    }
}
impl TryFrom<usize> for ActivePage {
    type Error = anyhow::Error;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ActivePage::Sensor),
            1 => Ok(ActivePage::Home),
            2 => Ok(ActivePage::Image),
            _ => Err(anyhow::anyhow!("Invalid ActivePage value: {value}")),
        }
    }
}
