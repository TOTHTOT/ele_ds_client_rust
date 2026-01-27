use crate::board::peripheral::{AllSensorData, Screen};
use crate::communication::weather::WeatherResponse;
use crate::device_config::DeviceConfig;
use crate::ui::home_page::HomePageInfo;
use crate::ui::image_page::ImagePageInfo;
use crate::ui::popup::PopupMsg;
use crate::ui::sensor_page::SensorPage;
use crate::{ui, ActivePage};
use anyhow::anyhow;
use mousefood::prelude::{Color, Frame, Rect, Style, Stylize, Terminal};
use mousefood::ratatui::widgets::canvas::{Context, Line, Rectangle};
use mousefood::ratatui::widgets::Block;
use mousefood::{fonts, EmbeddedBackend, EmbeddedBackendConfig};
use ssd1680::prelude::Display;
use std::sync::{Arc, Mutex};

macro_rules! hw_try {
    ($e:expr, $msg:expr) => {
        $e.map_err(|e| anyhow::anyhow!("{}: {:?}", $msg, e))?
    };
}
pub mod home_page;
mod image_page;
pub mod popup;
pub mod sensor_page;

type RenderClosure<'a> = Box<dyn FnOnce(&mut Frame) + 'a>;

#[derive(Default)]
pub struct UiInfo {
    pub net_state: bool,
    pub battery: u8,
}

/// 屏幕事件,
pub enum ScreenEvent {
    Refresh(ActivePage),
    UpdateSensorsData(AllSensorData),
    Popup(PopupMsg),
}

pub fn mouse_food_test(
    screen: &mut Screen,
    device_config: Arc<Mutex<DeviceConfig>>,
    set_active_page: ActivePage,
    popup_msg: Option<PopupMsg>,
) -> anyhow::Result<()> {
    if set_active_page == screen.current_page && !set_active_page.cur_set_page_is_need_refresh() {
        return Ok(());
    }
    display_select_page(screen, set_active_page, device_config, popup_msg)?;

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

/// 显示选中的页面
fn display_select_page(
    screen: &mut Screen,
    set_active_page: ActivePage,
    device_config: Arc<Mutex<DeviceConfig>>,
    popup_msg: Option<PopupMsg>,
) -> anyhow::Result<()> {
    let config = EmbeddedBackendConfig {
        font_regular: fonts::MONO_6X13,
        ..Default::default()
    };
    let backend = EmbeddedBackend::new(&mut screen.bw_buf, config);
    let mut terminal = Terminal::new(backend)?;

    let func = get_display_func(screen.last_sensor_status, set_active_page, device_config)?;
    terminal.draw(|f| {
        func(f);
        if let Some(popup_msg) = popup_msg {
            popup_msg.show_popup(f, None);
        }
    })?;
    Ok(())
}

/// 返回显示的页面, display_select_page() 使用
fn get_display_func<'d>(
    last_sensor_status: Option<AllSensorData>,
    set_active_page: ActivePage,
    device_config: Arc<Mutex<DeviceConfig>>,
) -> anyhow::Result<RenderClosure<'d>> {
    let mut device_config = device_config
        .lock()
        .map_err(|_| anyhow!("Mutex lock error"))?;
    let weather_response = device_config
        .weather
        .clone()
        .unwrap_or(WeatherResponse::default());
    device_config.current_page = set_active_page;
    let ui_info = UiInfo {
        net_state: false,
        battery: 10,
    };

    let func: RenderClosure = match set_active_page {
        ActivePage::Sensor => {
            let mut sensor = SensorPage {
                sensor_data: last_sensor_status.unwrap_or_default(),
                ui_info,
            };
            Box::new(move |f| sensor.sensor_page(f))
        }
        ActivePage::Home => {
            let mut home = HomePageInfo {
                weather_info: weather_response
                    .get_ui_need_data()
                    .unwrap_or(Default::default()),
                city: device_config.city_name_show.clone(),
                ui_info,
            };
            Box::new(move |f| home.home_page(f))
        }
        ActivePage::Image => {
            let mut image = ImagePageInfo {
                image_path: "/fat/system/images/test.bmp".to_string(),
                ui_info,
            };
            Box::new(move |f| image.image_page(f))
        }
        _ => anyhow::bail!("Not find selected page: {set_active_page:?}"),
    };
    Ok(func)
}

/// 父页面, 包含基本状态显示
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

// 绘制 大数字 的函数 使用 canvas
pub(crate) fn draw_big_digit(
    ctx: &mut Context,
    x_offset: f64,
    y_offset: f64,
    digit: char,
    w: f64,
    h: f64,
) {
    let color = Color::Black;

    let top = Line {
        x1: x_offset,
        y1: y_offset + h,
        x2: x_offset + w,
        y2: y_offset + h,
        color,
    };
    let mid = Line {
        x1: x_offset,
        y1: y_offset + h / 2.0,
        x2: x_offset + w,
        y2: y_offset + h / 2.0,
        color,
    };
    let bottom = Line {
        x1: x_offset,
        y1: y_offset,
        x2: x_offset + w,
        y2: y_offset,
        color,
    };
    let left_t = Line {
        x1: x_offset,
        y1: y_offset + h,
        x2: x_offset,
        y2: y_offset + h / 2.0,
        color,
    };
    let left_b = Line {
        x1: x_offset,
        y1: y_offset + h / 2.0,
        x2: x_offset,
        y2: y_offset,
        color,
    };
    let right_t = Line {
        x1: x_offset + w,
        y1: y_offset + h,
        x2: x_offset + w,
        y2: y_offset + h / 2.0,
        color,
    };
    let right_b = Line {
        x1: x_offset + w,
        y1: y_offset + h / 2.0,
        x2: x_offset + w,
        y2: y_offset,
        color,
    };

    match digit {
        '0' => {
            ctx.draw(&top);
            ctx.draw(&bottom);
            ctx.draw(&left_t);
            ctx.draw(&left_b);
            ctx.draw(&right_t);
            ctx.draw(&right_b);
        }
        '1' => {
            ctx.draw(&right_t);
            ctx.draw(&right_b);
        }
        '2' => {
            ctx.draw(&top);
            ctx.draw(&mid);
            ctx.draw(&bottom);
            ctx.draw(&right_t);
            ctx.draw(&left_b);
        }
        '3' => {
            ctx.draw(&top);
            ctx.draw(&mid);
            ctx.draw(&bottom);
            ctx.draw(&right_t);
            ctx.draw(&right_b);
        }
        '4' => {
            ctx.draw(&mid);
            ctx.draw(&left_t);
            ctx.draw(&right_t);
            ctx.draw(&right_b);
        }
        '5' => {
            ctx.draw(&top);
            ctx.draw(&mid);
            ctx.draw(&bottom);
            ctx.draw(&left_t);
            ctx.draw(&right_b);
        }
        '6' => {
            ctx.draw(&top);
            ctx.draw(&mid);
            ctx.draw(&bottom);
            ctx.draw(&left_t);
            ctx.draw(&left_b);
            ctx.draw(&right_b);
        }
        '7' => {
            ctx.draw(&top);
            ctx.draw(&right_t);
            ctx.draw(&right_b);
        }
        '8' => {
            ctx.draw(&top);
            ctx.draw(&mid);
            ctx.draw(&bottom);
            ctx.draw(&left_t);
            ctx.draw(&left_b);
            ctx.draw(&right_t);
            ctx.draw(&right_b);
        }
        '9' => {
            ctx.draw(&top);
            ctx.draw(&mid);
            ctx.draw(&bottom);
            ctx.draw(&left_t);
            ctx.draw(&right_t);
            ctx.draw(&right_b);
        }
        ':' => {
            let dot_size = 2.0;
            let x_pos = x_offset + w / 2.0 - dot_size / 2.0;
            ctx.draw(&Rectangle {
                x: x_pos,
                y: y_offset + h * 0.7 - dot_size / 2.0,
                width: dot_size,
                height: dot_size,
                color,
            });

            ctx.draw(&Rectangle {
                x: x_pos,
                y: y_offset + h * 0.3 - dot_size / 2.0,
                width: dot_size,
                height: dot_size,
                color,
            });
        }
        _ => {}
    }
}

/// 绘制canvas, 大数值, 实际渲染到画布中
pub(crate) fn canvas_draw_current_time(ctx: &mut Context) {
    let now = chrono::Local::now();
    let time_str: Vec<char> = now.format("%H:%M").to_string().chars().collect();
    // let time_str: Vec<char> = "000:000".chars().collect();

    // 渲染数字的大小, 随外部容器大小变化, 这里只是百分比
    let w = 12.0;
    let h = 60.0;
    let y_offset = (100.0 - h) / 2.0;
    let standard_spacing = 8.0;

    let mut total_content_width = 0.0;
    for idx in 0..time_str.len() {
        let c = time_str[idx];
        let char_w = if c == ':' { w / 3.0 } else { w };
        total_content_width += char_w;

        if idx < time_str.len() - 1 {
            let c_next = time_str.get(idx + 1);
            let next_spacing = if c == ':' || c_next == Some(&':') {
                standard_spacing * 0.8
            } else {
                standard_spacing
            };
            total_content_width += next_spacing;
        }
    }
    // 水平居中, 理论上应该渲染的起始位置
    let mut current_x = (100.0 - total_content_width) / 2.0;

    for idx in 0..time_str.len() {
        let c = time_str[idx];
        let char_w = if c == ':' { w / 3.0 } else { w };

        ui::draw_big_digit(ctx, current_x, y_offset, c, char_w, h);

        let c_next = time_str.get(idx + 1);
        let next_spacing = if c == ':' || c_next == Some(&':') {
            standard_spacing * 0.8
        } else {
            standard_spacing
        };
        current_x += char_w + next_spacing;
    }
}
