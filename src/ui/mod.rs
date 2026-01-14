use crate::board::peripheral::{ActivePage, Screen};
use crate::ui::home_page::HomePageInfo;
use crate::ui::sensor_page::SensorPage;
use anyhow::anyhow;
use mousefood::prelude::{Frame, Rect, Style, Stylize};
use mousefood::ratatui::widgets::Block;
use ssd1680::prelude::Display;
use std::sync::{Arc, Mutex};

pub mod home_page;
pub mod sensor_page;
#[derive(Default)]

pub struct UiInfo {
    home: HomePageInfo,
    sensor: SensorPage,
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
    let pages = [HomePageInfo::build_home_page, SensorPage::build_sensor_page];
    let mut screen = screen.lock().map_err(|_| anyhow!("Mutex lock error"))?;
    if set_active_page == screen.current_page {
        return Ok(());
    }
    screen.current_page = set_active_page;
    let page = pages
        .get(screen.current_page as usize)
        .ok_or(anyhow!("Page not found"))?;
    page(&mut screen)?;
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
