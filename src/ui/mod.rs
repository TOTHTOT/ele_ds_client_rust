use crate::board::peripheral::{AllSensorData, Screen};
use crate::communication::weather::WeatherResponse;
use crate::device_config::DeviceConfig;
use crate::ui::home_page::HomePageInfo;
use crate::ui::image_page::ImagePageInfo;
use crate::ui::sensor_page::SensorPage;
use crate::ActivePage;
use anyhow::anyhow;
use mousefood::prelude::{Frame, Rect, Style, Stylize};
use mousefood::ratatui::widgets::Block;
use ssd1680::prelude::Display;
use std::sync::{Arc, Mutex};

pub mod home_page;
mod image_page;
pub mod sensor_page;

#[derive(Default)]
pub struct UiInfo {
    pub net_state: bool,
    pub battery: u8,
}

/// 屏幕事件,
pub enum ScreenEvent {
    Refresh(ActivePage),
    UpdateSensorsData(AllSensorData),
}

/// 用于包裹 ssd1680返回的错误
macro_rules! hw_try {
    ($e:expr, $msg:expr) => {
        $e.map_err(|e| anyhow::anyhow!("{}: {:?}", $msg, e))?
    };
}

pub fn mouse_food_test(
    screen: &mut Screen,
    device_config: Arc<Mutex<DeviceConfig>>,
    set_active_page: ActivePage,
) -> anyhow::Result<()> {
    if set_active_page == screen.current_page && !set_active_page.cur_set_page_is_need_refresh() {
        return Ok(());
    }
    let mut config = device_config
        .lock()
        .map_err(|_| anyhow!("Mutex lock error"))?;
    let weather_str = config.weather.clone().unwrap_or(WeatherResponse::default());

    let ui_info = UiInfo {
        net_state: false,
        battery: 10,
    };
    match set_active_page {
        ActivePage::Sensor => {
            let mut sensor = SensorPage {
                sensor_data: screen.last_sensor_status.unwrap_or_default(),
                ui_info,
            };
            SensorPage::build_sensor_page(screen, &mut sensor)?;
        }
        ActivePage::Home => {
            let mut home = HomePageInfo {
                weather_info: weather_str.get_ui_need_data()?,
                city: config.city_name.to_string(),
                ui_info,
            };
            HomePageInfo::build_home_page(screen, &mut home)?;
        }
        ActivePage::Image => {
            let mut image = ImagePageInfo {
                image_path: "/fat/system/images/test.bmp".to_string(),
                ui_info,
            };
            ImagePageInfo::build_image_page(screen, &mut image)?;
        }
        _ => anyhow::bail!("Not find selected page: {set_active_page:?}"),
    }
    config.current_page = set_active_page;
    drop(config);
    screen.current_page = set_active_page;
    // 单独解构这些, 避免借用问题
    let Screen {
        ref mut ssd1680,
        ref mut delay,
        ref mut bw_buf,
        ..
    } = &mut *screen;

    hw_try!(ssd1680.init(delay), "Ssd1680 init");
    hw_try!(ssd1680.update_bw_frame(bw_buf.buffer()), "Ssd1680 update");
    hw_try!(ssd1680.display_frame(delay), "Ssd1680 display");
    hw_try!(ssd1680.entry_sleep(), "Ssd1680 sleep");
    Ok(())
}

pub(super) fn general_block(f: &mut Frame, info: &UiInfo) -> Rect {
    let outer_block = Block::bordered()
        .border_style(Style::new().black())
        .title(format!(
            " Net: {} Battery: {}% ",
            if info.net_state {
                "Connect"
            } else {
                "Disconnect"
            },
            info.battery
        ));
    let main_area = outer_block.inner(f.area());
    f.render_widget(outer_block, f.area());
    main_area
}
