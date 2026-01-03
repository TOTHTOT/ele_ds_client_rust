use crate::board::BoardPeripherals;
use crate::ui::{general_block, UiInfo};
use anyhow::anyhow;
use mousefood::prelude::{Alignment, Constraint, Direction, Frame, Layout, Terminal};
use mousefood::ratatui::widgets::Paragraph;
use mousefood::{fonts, EmbeddedBackend, EmbeddedBackendConfig};
use std::sync::{Arc, Mutex};

pub struct SensorPage {
    temp: f32,
    humi: f32,
    press: u16,
    lux: u16,
}
impl Default for SensorPage {
    fn default() -> SensorPage {
        Self {
            temp: 0.0,
            humi: 0.0,
            press: 0,
            lux: 0,
        }
    }
}
impl SensorPage {
    pub fn build_sensor_page(board: Arc<Mutex<BoardPeripherals>>) -> anyhow::Result<()> {
        {
            let mut board = board.lock().map_err(|_| anyhow!("Mutex lock error"))?;
            let config = EmbeddedBackendConfig {
                font_regular: fonts::MONO_6X13,
                ..Default::default()
            };
            let backend = EmbeddedBackend::new(&mut board.bw_buf, config);
            let mut terminal = Terminal::new(backend)?;
            terminal.draw(|f| Self::sensor_page(f, UiInfo::default()))?;
        }
        Ok(())
    }
    fn sensor_page(f: &mut Frame, info: UiInfo) {
        let main_area = general_block(f, &info.home);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1) // 留出一点边距防止贴边
            .constraints([
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
            ])
            .split(main_area);

        let sensors = [
            ("TEMP", format!("{:.1} C", info.sensor.temp)),
            ("HUMI", format!("{:.1} %", info.sensor.humi)),
            ("PRES", format!("{:.0} hPa", info.sensor.press)),
            ("LUX ", format!("{} lx", info.sensor.lux)),
        ];

        for (i, (label, value)) in sensors.iter().enumerate() {
            let item_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(6), // 标签固定宽度
                    Constraint::Min(10),   // 数值占用剩余空间
                ])
                .split(chunks[i]);

            f.render_widget(
                Paragraph::new(format!("[{label}]",)).alignment(Alignment::Left),
                item_chunks[0],
            );

            f.render_widget(
                Paragraph::new(value.as_str()).alignment(Alignment::Right),
                item_chunks[1],
            );
        }
    }
}
