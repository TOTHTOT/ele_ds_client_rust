use crate::board::peripheral::AllSensorData;
use crate::ui;
use crate::ui::{general_block, UiInfo};
use mousefood::prelude::Frame;
use std::default::Default;

#[derive(Default)]
pub struct SensorPage {
    pub sensor_data: AllSensorData,
    pub ui_info: UiInfo,
}
impl SensorPage {
    pub fn sensor_page(&mut self, f: &mut Frame) {
        let main_area = general_block(f, &self.ui_info);

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
        ui::show_contents_from_chunks(f, main_area, &sensors);
    }
}
