use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceInfo {
    version: String,
    device_type: String,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            version: String::from(env!("BUILD_TIME")),
            device_type: String::from("ele_ds_client_rust"),
        }
    }
}
