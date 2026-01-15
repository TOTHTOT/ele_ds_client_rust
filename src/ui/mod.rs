use crate::board::peripheral::{AllSensorData, Screen};
use crate::communication::weather::Weather;
use crate::ui::home_page::HomePageInfo;
use crate::ui::sensor_page::SensorPage;
use crate::ActivePage;
use anyhow::anyhow;
use chrono::Timelike;
use mousefood::prelude::{Frame, Rect, Style, Stylize};
use mousefood::ratatui::widgets::Block;
use ssd1680::prelude::Display;
use std::sync::{Arc, Mutex, MutexGuard};

pub mod home_page;
pub mod sensor_page;
#[derive(Default)]
pub struct UiInfo {
    pub home: HomePageInfo,
    pub sensor: SensorPage,
}

/// 用于包裹 ssd1680返回的错误
macro_rules! hw_try {
    ($e:expr, $msg:expr) => {
        $e.map_err(|e| anyhow::anyhow!("{}: {:?}", $msg, e))?
    };
}

pub fn mouse_food_test(
    screen: Arc<Mutex<Screen>>,
    set_active_page: ActivePage,
) -> anyhow::Result<()> {
    let mut screen = screen.lock().map_err(|_| anyhow!("Mutex lock error"))?;
    if set_active_page == screen.current_page && !set_active_page.cur_set_page_is_need_refresh() {
        return Ok(());
    }
    let home = HomePageInfo {
        net_state: false,
        weather_info: get_weather_per_hour(&mut screen)?,
        battery: 10,
        city: "Fuzhou".to_string(),
    };
    let sensor = SensorPage {
        sensor_data: screen
            .last_sensor_status
            .unwrap_or(AllSensorData::default()),
    };
    let mut info = UiInfo { home, sensor };
    match set_active_page {
        ActivePage::Sensor => SensorPage::build_sensor_page(&mut screen, &mut info)?,
        ActivePage::Home => HomePageInfo::build_home_page(&mut screen, &mut info)?,
        ActivePage::Image => anyhow::bail!("not support now"),
        _ => anyhow::bail!("Not find selected page: {set_active_page:?}"),
    }
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
    drop(screen);
    Ok(())
}

/// 每小时更新一次时间, 默认都返回 default_data , 除非 get_ui_need_data()失败
fn get_weather_per_hour(screen: &mut MutexGuard<Screen>) -> anyhow::Result<[String; 3]> {
    let now = chrono::Local::now().hour();
    let default_data = [
        "Sunny 25℃".to_string(),
        "Sunny 25℃".to_string(),
        "Sunny 25℃".to_string(),
    ];
    if now != screen.last_hour {
        if let Ok(weather) = Weather::new(
            &screen.device_config.as_ref().city_name,
            &screen.device_config.as_ref().weather_api_key,
        )
        .get_weather_hefeng()
        {
            screen.last_hour = now;
            Ok(weather.get_ui_need_data()?)
        } else {
            Ok(default_data)
        }
    } else {
        Ok(default_data)
    }
}

pub(super) fn general_block(f: &mut Frame, info: &HomePageInfo) -> Rect {
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
