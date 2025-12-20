use crate::device_info::DeviceInfo;
use crate::ele_ds_http_client::EleDsHttpClient;
use esp_idf_svc::io;
use esp_idf_svc::ota::EspOta;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

const REQUERY_WHETHER_UPGRADE: &str = "/upgrade/ele_ds_client/requery";
const GET_UPGRADE_FILE: &str = "/upgrade/ele_ds_client/";
pub struct Ota {
    http_client: Arc<Mutex<EleDsHttpClient>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct VersionInfo {
    version: String,
}

impl Ota {
    pub fn new(http_client: Arc<Mutex<EleDsHttpClient>>) -> anyhow::Result<Self> {
        Ok(Ota { http_client })
    }
    pub fn is_need_upgrade(&self) -> anyhow::Result<bool> {
        let device_info = serde_json::to_string(&DeviceInfo::default())?;
        let mut client = self
            .http_client
            .lock()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let (statue, response_json) = client.post_msg(REQUERY_WHETHER_UPGRADE, &device_info)?;
        log::info!("statue: {}, response: {}", statue, response_json);
        let version_info: VersionInfo = serde_json::from_str(&response_json)?;
        if version_info.version.len() > 5 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /*fn read_firmware_info(
        response: &mut Response<&mut EspHttpConnection>,
    ) -> anyhow::Result<FirmwareInfo> {
        let mut update = ota.initiate_update()?;

        let mut buffer = [0_u8; 4096];
        let update_info_load = EspFirmwareInfoLoad {};
        let mut update_info = FirmwareInfo {
            version: Default::default(),
            released: Default::default(),
            description: Default::default(),
            signature: Default::default(),
            download_id: Default::default(),
        };

        loop {
            let n = response.read(&mut buffer)?;
            update.write(&buffer[0..n])?;
            if update_info_load.fetch(&buffer[0..n], &mut update_info)? {
                return Ok(update_info);
            }
        }
    }*/

    pub fn get_upgrade_file(&self, file_name: &str) -> anyhow::Result<()> {
        let mut client = self
            .http_client
            .lock()
            .map_err(|e| anyhow::anyhow!("get_upgrade_file() lock failed: {e}"))?;
        client.get_file(
            format!("{}{}", GET_UPGRADE_FILE, file_name).as_str(),
            move |response| {
                let mut buffer = [0_u8; 1024];
                let mut ota = EspOta::new().expect("Failed to create ota client");
                let update = ota.initiate_update().expect("failed to initiate update");
                if let Err(e) = io::utils::copy(response, update, &mut buffer) {
                    log::error!("get_upgrade_file() copy upgrade file failed, {e}");
                }
            },
        )?;
        Ok(())
    }
}
