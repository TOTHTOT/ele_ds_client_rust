use crate::device_info;
use crate::ele_ds_http_client::EleDsHttpClient;
use esp_idf_svc::ota::EspOta;
use std::sync::{Arc, Mutex};
use crate::device_info::DeviceInfo;

pub struct Ota {
    http_client: Arc<Mutex<EleDsHttpClient>>,
}

impl Ota {
    pub fn new(http_client: Arc<Mutex<EleDsHttpClient>>) -> Self {
        Ota { http_client }
    }
    pub fn is_need_upgrade(&self) -> anyhow::Result<bool> {
        let device_info = serde_json::to_string(&DeviceInfo::default())?;
        let mut client = self
            .http_client
            .lock()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let (statue, response_json) = client.post_msg(&device_info)?;
        log::info!("statue: {}, response: {}", statue, response_json);
        Ok(true)
    }
    pub fn ota_upgrade() -> anyhow::Result<()> {
        let mut ota = EspOta::new()?;

        let mut update = ota.initiate_update()?;
        // 3. Write the program data:
        // while let Some(data) = my_wireless.get_ota_data() {
        //     update.write(&data).map_err(|_| anyhow::anyhow!("ota data error"))?;
        // }

        // update
        //         .complete()?;

        esp_idf_svc::hal::reset::restart();

        Ok(())
    }
}
