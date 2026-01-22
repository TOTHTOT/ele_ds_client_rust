use crate::board::peripheral::Screen;
use crate::ui::{general_block, UiInfo};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::{image::Image, prelude::*};
use mousefood::prelude::*;
use mousefood::ratatui::widgets::canvas::{Canvas, Context, Line};
use mousefood::ratatui::widgets::{Block, Paragraph, Wrap};
use mousefood::{fonts, EmbeddedBackend};
use ssd1680::graphics::DisplayAnyIn;
use tinybmp::Bmp;

pub struct HomePageInfo {
    pub weather_info: [String; 3],
    pub city: String,
    pub ui_info: UiInfo,
}
impl Default for HomePageInfo {
    fn default() -> Self {
        Self {
            weather_info: ["".to_string(), "".to_string(), "".to_string()],
            city: "Fuzhou".to_string(),
            ui_info: UiInfo::default(),
        }
    }
}

#[allow(dead_code)]
impl HomePageInfo {
    pub fn build_home_page(screen: &mut Screen, info: &mut HomePageInfo) -> anyhow::Result<()> {
        {
            let config = EmbeddedBackendConfig {
                font_regular: fonts::MONO_6X13,
                ..Default::default()
            };
            let backend = EmbeddedBackend::new(&mut screen.bw_buf, config);
            let mut terminal = Terminal::new(backend)?;
            terminal.draw(|f| info.home_page(f))?;
        }
        // Self::pad_time_date_image(&mut screen.bw_buf)?;
        Ok(())
    }

    pub fn home_page(&mut self, f: &mut Frame) {
        let main_area = general_block(f, &self.ui_info);

        // 将内部区域垂直切分割
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

        f.render_widget(
            Canvas::default()
                .x_bounds([0.0, 100.0])
                .y_bounds([0.0, 100.0])
                .paint(|ctx| {
                    let now = chrono::Local::now();
                    let time_str = now.format("%H:%M").to_string();
                    let w = 5.0 * 2.0;
                    let h = 30.0 * 2.0;
                    let y_offset = (100.0 - h) / 2.0;
                    for (i, c) in time_str.chars().enumerate() {
                        Self::draw_big_digit(ctx, (10.0 + w) * i as f64, y_offset, c, w, h);
                    }
                }),
            clock_chunks[0],
        );
        let now = chrono::Local::now();
        let date_str = now.format("%Y/%m/%d %a").to_string();
        f.render_widget(
            Paragraph::new(date_str)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            clock_chunks[1],
        );
        let get_w = |i: usize| self.weather_info.get(i).map_or("Null", |m| m.as_str());
        let weather_content = format!("Today: \n{}\nTomorrow:\n{}", get_w(0), get_w(1));
        f.render_widget(
            Paragraph::new(weather_content)
                .alignment(Alignment::Left)
                .block(Block::bordered().title(format!(" {} Weather ", self.city))),
            main_chunks[1],
        );
    }

    // 绘制 大数字 的函数 使用 canvas
    fn draw_big_digit(
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
                // 画两个点
                ctx.print(x_offset + w / 1.0, y_offset + h * 0.7, ".");
                ctx.print(x_offset + w / 1.0, y_offset + h * 0.3, ".");
            }
            _ => {}
        }
    }

    /// 由于需要显示很大的时间但是 mousefood 不能单独设置字体, 这里只能在外部根据坐标填充时间,
    /// 这里是图片显示的方法, 在实现 popup 后发现会覆盖 popup 效果很糟糕
    pub(crate) fn pad_time_date_image(buf: &mut DisplayAnyIn) -> anyhow::Result<()> {
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
                    img.draw(buf)
                        .map_err(|e| anyhow::anyhow!("pad_time_date filed: {e:?}"))?;
                    current_x += (bmp.size().width + 4) as i32;
                }
            }
        }
        Ok(())
    }
}
