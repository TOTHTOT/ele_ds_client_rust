// src/lib.rs

use serde::{Deserialize, Serialize};

pub mod board;
pub mod cmd_menu;
pub mod communication;
pub mod device_config;
pub mod file_system;
pub mod ui;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(usize)]
pub enum ActivePage {
    None,
    Sensor,
    #[default]
    Home,
    Image,
    Setting,
    About,
}

impl ActivePage {
    /// 当前页面是否需要在每次收到更新命令时刷新
    pub fn cur_set_page_is_need_refresh(self) -> bool {
        if self == ActivePage::Home || self == ActivePage::Sensor {
            return true;
        }
        false
    }
}
impl TryFrom<usize> for ActivePage {
    type Error = anyhow::Error;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ActivePage::Sensor),
            1 => Ok(ActivePage::Home),
            2 => Ok(ActivePage::Image),
            _ => Err(anyhow::anyhow!("Invalid ActivePage value: {value}")),
        }
    }
}

pub const SERVER_CERT: &str = "-----BEGIN CERTIFICATE-----\n\
MIIDYjCCAkqgAwIBAgIUW8aMRyWSarT0jjgQTlHzlRbtVQEwDQYJKoZIhvcNAQEL\n\
BQAwJDEiMCAGA1UEAwwZd3d3LWZ1bi51MTc4NjQ2NS5ueWF0LmFwcDAeFw0yNTEy\n\
MjEwNDU3MThaFw0zNTEyMTkwNDU3MThaMCQxIjAgBgNVBAMMGXd3dy1mdW4udTE3\n\
ODY0NjUubnlhdC5hcHAwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQCp\n\
wiA9mfcER3oeElVenaJMEAhkjhDysGynlEn91ys9ysONAslCT5paOHuzhCgHzJ50\n\
GGA8rDksvywv0PnAJfo1xkixkxFELM5m+wJ2VlxoiyHm2IK+xGiS6OyDVQqMY+q0\n\
f/5ygvPbEvX+6UYE0D1E+l5i4aLGzybWgIfR82+RHlBDJh5hrEuOFloJakugxvEC\n\
qqmQMrCb0vkVGM94OfdDqonDI9cl7woaet4ahkwRjoXNAVXhmpi9+Bn/0Bdkbz1k\n\
agR/DJArdNi41yUYKmnX1+HJTwMZdtx+kn8AdsMIYj0V0LZ9ch0IHGcQRHVbJ3Ae\n\
OvW2E/oj826ya+RtthPzAgMBAAGjgYswgYgwHQYDVR0OBBYEFD6YNMib61v5Htg2\n\
+97lDDySZcDqMB8GA1UdIwQYMBaAFD6YNMib61v5Htg2+97lDDySZcDqMA8GA1Ud\n\
EwEB/wQFMAMBAf8wNQYDVR0RBC4wLIIZd3d3LWZ1bi51MTc4NjQ2NS5ueWF0LmFw\n\
cIIJbG9jYWxob3N0hwQ814BJMA0GCSqGSIb3DQEBCwUAA4IBAQBtBOQOMIOvRlRJ\n\
rOylO+MgrXsUGmkc4Y2rmJijwRvJXPFp7vDEV+9U27rOOuld5X0qzp13WFygvWSi\n\
7ahobCCKubwDD3jowqYOfWzQA64knkYa7BV7qc0KoTd9K7RU8a8myzjA00K0O50B\n\
YF3zwfvf7W3d2Dia1wrcHStzgxxrQ855LON+k2C0mm2cTO2Z/INULAi7/g2g4vgP\n\
2f8adk8AXz4G8+w2PwCqqHR2Ckv0WDSb7OmTPSFEXsNfwqcM8UD2w/LSzptvfha3\n\
qFhlHvAAsDNLmmCumqZBfapQpzVfF4Y2S2nWYc1YFei4a/bumLTw4WOGhip0nSD7\n\
9eBZPwgY\n\
-----END CERTIFICATE-----";
