use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceInfo {
    version: String,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            version: String::from(env!("BUILD_TIME")),
        }
    }
}
