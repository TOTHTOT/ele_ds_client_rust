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

    FullTime,    // 双击左边按键
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

    /// 有的界面只需要短时间显示, 下一个周期就刷新到home
    pub fn cur_page_is_not_need_record(self) -> bool {
        self == ActivePage::About || self == ActivePage::Setting
    }

    /// 键值映射成页面
    pub fn from_event(button_idx: usize, click_count: u8) -> Self {
        match (click_count, button_idx) {
            (1, 0) => ActivePage::Sensor,
            (1, 1) => ActivePage::Home,
            (1, 2) => ActivePage::Image,

            (2, 0) => ActivePage::FullTime,
            (2, 1) => ActivePage::Setting,
            (2, 2) => ActivePage::FullWeather,

            (3, 1) => ActivePage::About,

            _ => ActivePage::None,
        }
    }
}
