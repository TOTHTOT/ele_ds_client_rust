use crate::board::peripheral::AllSensorData;
use crate::ui::{general_block, Screen, UiInfo};
use mousefood::prelude::{Alignment, Constraint, Direction, Frame, Layout, Terminal};
use mousefood::ratatui::widgets::Paragraph;
use mousefood::{fonts, EmbeddedBackend, EmbeddedBackendConfig};
use std::default::Default;

#[derive(Default)]
pub struct SensorPage {
    pub sensor_data: AllSensorData,
}
impl SensorPage {
    pub fn build_sensor_page(screen: &mut Screen, info: &mut UiInfo) -> anyhow::Result<()> {
        {
            let config = EmbeddedBackendConfig {
                font_regular: fonts::MONO_6X13,
                ..Default::default()
            };
            let backend = EmbeddedBackend::new(&mut screen.bw_buf, config);
            let mut terminal = Terminal::new(backend)?;
            terminal.draw(|f| Self::sensor_page(f, info))?;
        }
        Ok(())
    }
    fn sensor_page(f: &mut Frame, info: &mut UiInfo) {
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
            (
                "TEMP",
                format!("{:.1} C", info.sensor.sensor_data.sht3x_measure.temperature),
            ),
            (
                "HUMI",
                format!("{:.1} %", info.sensor.sensor_data.sht3x_measure.humidity),
            ),
            ("PRES", format!("{:.0} hPa", 0)),
            ("LUX ", format!("{} lx", 0)),
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
