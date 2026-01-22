use crate::board::peripheral::AllSensorData;
use crate::ui::{general_block, UiInfo};
use mousefood::prelude::{Alignment, Constraint, Direction, Frame, Layout};
use mousefood::ratatui::widgets::Paragraph;
use std::default::Default;

#[derive(Default)]
pub struct SensorPage {
    pub sensor_data: AllSensorData,
    pub ui_info: UiInfo,
}
impl SensorPage {
    // pub fn build_sensor_page(screen: &mut Screen, info: &mut SensorPage) -> anyhow::Result<()> {
    //     {
    //         let config = EmbeddedBackendConfig {
    //             font_regular: fonts::MONO_6X13,
    //             ..Default::default()
    //         };
    //         let backend = EmbeddedBackend::new(&mut screen.bw_buf, config);
    //         let mut terminal = Terminal::new(backend)?;
    //         terminal.draw(|f| Self::sensor_page(f, info))?;
    //     }
    //     Ok(())
    // }
    pub fn sensor_page(&mut self, f: &mut Frame) {
        let main_area = general_block(f, &self.ui_info);

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
                format!("{:.1} C", self.sensor_data.sht3x_measure.temperature),
            ),
            (
                "HUMI",
                format!("{:.1} %", self.sensor_data.sht3x_measure.humidity),
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
