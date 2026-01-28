use crate::ui::{general_block, show_contents_from_chunks, UiInfo};
use mousefood::prelude::Frame;
use std::default::Default;

#[derive(Default)]
pub struct AboutPage {
    pub ip_addr: String,
    pub connect_wifi: String,
    pub wifi_password: String,
    pub soft_version: String,
    pub ui_info: UiInfo,
}
impl AboutPage {
    pub fn about_page(&mut self, f: &mut Frame) {
        let main_area = general_block(f, &self.ui_info);

        let contents = [
            ("wifi ssid", self.connect_wifi.to_string()),
            ("wifi password", self.wifi_password.to_string()),
            ("ip addr", self.ip_addr.to_string()),
            ("ble name", self.ip_addr.to_string()),
            ("soft version", self.soft_version.to_string()),
        ];
        show_contents_from_chunks(f, main_area, &contents);
    }
}
