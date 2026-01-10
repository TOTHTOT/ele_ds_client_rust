use crate::board::peripheral::BoardPeripherals;
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

pub fn mouse_food_test(board: &mut Arc<Mutex<BoardPeripherals>>) -> anyhow::Result<()> {
    let pages = vec![
        HomePageInfo::build_home_page, /*, SensorPage::build_sensor_page*/
    ];
    for page in pages {
        page(board.clone())?;
        let mut board = board.lock().map_err(|_| anyhow!("Mutex lock error"))?;
        // 单独解构这些, 避免借用问题
        let BoardPeripherals {
            ref mut ssd1680,
            ref mut bw_buf,
            ref mut delay,
            ..
        } = &mut *board;
        let buffer_data = bw_buf.buffer();
        ssd1680.update_bw_frame(buffer_data).unwrap();
        ssd1680.display_frame(delay).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
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
