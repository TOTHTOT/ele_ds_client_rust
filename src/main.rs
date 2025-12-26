use ele_ds_client_rust::{
    board::BoardPeripherals,
    communication::{http_client, ota},
};
use std::sync::{Arc, Mutex};
fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("system start, build info: {} 12", env!("BUILD_TIME"));
    let _board = BoardPeripherals::new()?;
    /*match wifi_connect(&mut wifi, "esp-2.4G", "12345678..") {
        Ok(_) => {
            if let Err(e) = after_wifi_established() {
                log::warn!("after_wifi_established() failed: {e}")
            }
        }
        Err(_) => log::warn!("failed to connect wifi"),
    }*/


    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        // ele_ds_client_rust::power_manage::enter_deep_sleep_mode();
    }
}

/// wifi 连接成功要做的一些内容
pub fn after_wifi_established() -> anyhow::Result<()> {
    // 创建http客户端
    let client = Arc::new(Mutex::new(http_client::EleDsHttpClient::new(
        "https://60.215.128.73:12675",
    )?));
    let client_ota = client.clone();
    let ota = ota::Ota::new(client_ota);
    match ota {
        Ok(ota) => {
            if let Err(e) = ota.sync_firmware() {
                log::error!("sync_firmware failed: {}", e);
            }
        }
        Err(e) => log::warn!("create ota failed, {:?}", e),
    }
    Ok(())
}
