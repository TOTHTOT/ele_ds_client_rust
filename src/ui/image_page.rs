use crate::board::peripheral::Screen;
use crate::ui;
use crate::ui::{general_block, UiInfo};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::{image::Image, prelude::*};
use mousefood::prelude::symbols::Marker;
use mousefood::prelude::{Color, Frame, Rect, Terminal, Widget};
use mousefood::ratatui::widgets::canvas::{Canvas, Map, MapResolution};
use mousefood::{fonts, EmbeddedBackend, EmbeddedBackendConfig};
use ssd1680::prelude::DisplayAnyIn;
use tinybmp::Bmp;

pub struct ImagePageInfo {
    pub image_path: String,
    pub(crate) ui_info: UiInfo,
}
impl Default for ImagePageInfo {
    fn default() -> Self {
        Self {
            image_path: "/fat/system/tmd.0.bmp".to_string(),
            ui_info: UiInfo::default(),
        }
    }
}
#[allow(dead_code)]
impl ImagePageInfo {
    pub fn build_image_page(screen: &mut Screen, info: &mut ImagePageInfo) -> anyhow::Result<()> {
        {
            let config = EmbeddedBackendConfig {
                font_regular: fonts::MONO_6X13,
                ..Default::default()
            };
            let backend = EmbeddedBackend::new(&mut screen.bw_buf, config);
            let mut terminal = Terminal::new(backend)?;
            terminal.draw(|f| info.image_page(f))?;
        }
        let main_area = Rect {
            x: 1,
            y: 1,
            width: 47,
            height: 7,
        };

        Self::pat_image(&mut screen.bw_buf, &info.image_path, main_area)?;
        Ok(())
    }

    pub fn image_page(&mut self, f: &mut Frame) {
        let main_area = general_block(f, &self.ui_info);
        log::info!("main_area: {main_area:?}");
        // f.render_widget(
        //     Canvas::default()
        //         .marker(Marker::HalfBlock)
        //         .x_bounds([0.0, 100.0])
        //         .y_bounds([0.0, 100.0])
        //         .paint(ui::canvas_draw_current_time),
        //     main_area,
        // );
    }

    pub fn pat_image(
        buf: &mut DisplayAnyIn,
        image_path: &str,
        main_area: Rect,
    ) -> anyhow::Result<()> {
        let raw_data = std::fs::read(image_path)?;

        let char_w = 6;
        let char_h = 13;

        match Bmp::<BinaryColor>::from_slice(&raw_data) {
            Ok(bmp) => {
                let px_area_x = main_area.x as i32 * char_w;
                let px_area_y = main_area.y as i32 * char_h;
                let px_area_w = main_area.width as i32 * char_w;
                let px_area_h = main_area.height as i32 * char_h;

                let img_w = bmp.as_raw().header().image_size.width as i32;
                let img_h = bmp.as_raw().header().image_size.height as i32;
                log::info!("bmp size: {:?}", bmp.as_raw().header().image_size);
                let x_offset = (px_area_w - img_w) / 2;
                let y_offset = (px_area_h - img_h) / 2;

                let image_pos = Point::new(px_area_x + x_offset, px_area_y + y_offset);

                log::info!("Drawing image at pixel pos: {image_pos:?}");

                let image = Image::new(&bmp, image_pos);
                image
                    .draw(buf)
                    .map_err(|e| anyhow::anyhow!("Draw error: {e:?}"))?;
            }
            Err(e) => {
                log::error!("BMP parse error: {e:?}");
            }
        }
        Ok(())
    }

    /// 测试canvas组件
    fn map_canvas_test() -> impl Widget + 'static {
        Canvas::default()
            .marker(Marker::Dot)
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::Black,
                    resolution: MapResolution::Low,
                });
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
    }
}
