use crate::board::BoardPeripherals;
use anyhow::anyhow;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::{image::Image, prelude::*};
use mousefood::prelude::*;
use mousefood::ratatui::widgets::{Block, Paragraph, Wrap};
use mousefood::{fonts, EmbeddedBackend};
use ssd1680::prelude::Display;
use std::sync::{Arc, Mutex};
use tinybmp::Bmp;

pub struct HomePageInfo {
    pub net_state: bool,
    pub weather_info: [String; 3],
    pub battery: u8,
}
impl Default for HomePageInfo {
    fn default() -> Self {
        Self {
            net_state: true,
            weather_info: [
                "Sunny 25℃".to_string(),
                "Sunny 25℃".to_string(),
                "Sunny 25℃".to_string(),
            ],
            battery: 100,
        }
    }
}

pub fn mouse_food_test(board: &mut Arc<Mutex<BoardPeripherals>>) -> anyhow::Result<()> {
    {
        let mut board = board.lock().map_err(|_| anyhow!("Mutex lock error"))?;
        let config = EmbeddedBackendConfig {
            font_regular: fonts::MONO_6X13,
            ..Default::default()
        };
        let backend = EmbeddedBackend::new(&mut board.bw_buf, config);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| home_page(f, HomePageInfo::default()))?;
    }
    let mut board = board.lock().map_err(|_| anyhow!("Mutex lock error"))?;
    pad_time_date(&mut board)?;
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
    Ok(())
}

fn home_page(f: &mut Frame, info: HomePageInfo) {
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

    // 获取外层 Block 内部的可用区域
    let main_area = outer_block.inner(f.area());
    f.render_widget(outer_block, f.area());

    // 将内部区域垂直切分为 Clock (60%) 和 Weather (40%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Clock 区域
            Constraint::Percentage(40), // Weather 区域
        ])
        .split(main_area);

    let clock = Block::bordered().title(" Clock ");
    let clock_inner_area = clock.inner(main_chunks[0]);
    f.render_widget(clock, main_chunks[0]);
    let clock_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70), // Time 区域
            Constraint::Percentage(30), // Date 区域
        ])
        .split(clock_inner_area);

    let now = chrono::Local::now();
    let date_str = now.format("%Y/%m/%d %a").to_string(); // 例如 01/03 Sat
    f.render_widget(
        Paragraph::new(date_str)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        clock_chunks[1],
    );
    let get_w = |i: usize| info.weather_info.get(i).map_or("Null", |m| m.as_str());
    let weather_content = format!("Today: \n    {}\nTomorrow:\n   {}", get_w(0), get_w(1));
    f.render_widget(
        Paragraph::new(weather_content)
            .alignment(Alignment::Left)
            .block(Block::bordered().title(" Weather ")),
        main_chunks[1],
    );
}

/// 由于需要显示很大的时间但是 mousefood 不能单独设置字体, 这里只能在外部根据坐标填充时间
fn pad_time_date(board: &mut BoardPeripherals) -> anyhow::Result<()> {
    let time_str = chrono::Local::now().format("%H:%M").to_string();
    let mut current_x = 25; // 这里的初始值决定了整体左右偏移
    let y_position = 35; // 这里的初始值决定了上下偏移

    for c in time_str.chars() {
        let path = if c == ':' {
            "/fat/system/tmd/colon.bmp".to_string()
        } else {
            format!("/fat/system/tmd/{c}.bmp")
        };

        if let Ok(data) = std::fs::read(&path) {
            if let Ok(bmp) = Bmp::<BinaryColor>::from_slice(&data) {
                let img = Image::new(&bmp, Point::new(current_x, y_position));
                img.draw(&mut board.bw_buf).unwrap();
                current_x += (bmp.size().width + 4) as i32;
            }
        }
    }

    Ok(())
}
