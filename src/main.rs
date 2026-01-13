use ele_ds_client_rust::board::power_manage::next_minute_left_time;
use ele_ds_client_rust::communication::http_server::HttpServer;
use ele_ds_client_rust::ui::mouse_food_test;
use ele_ds_client_rust::{
    board::peripheral::BoardPeripherals,
    communication::{http_client, ota},
};
use std::sync::{Arc, Mutex};

#[allow(clippy::arc_with_non_send_sync)]
fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("system start, build info: {} 12", env!("BUILD_TIME"));
    let mut board = BoardPeripherals::new()?;
    board.device_config.boot_times_add()?;
    let board = Arc::new(Mutex::new(board));
    let mut ui_board = board.clone();

    let _http_server = HttpServer::new()?;

    loop {
        {
            let mut board = board
                .lock()
                .map_err(|e| anyhow::anyhow!("lock board failed: {e:?}"))?;
            let device_status = board.read_all_sensor()?;
            log::info!("device_status: {device_status:?}");
        }
        mouse_food_test(&mut ui_board)?;
        std::thread::sleep(std::time::Duration::from_micros(next_minute_left_time()));
        // ele_ds_client_rust::power_manage::enter_deep_sleep_mode_per_minute();
    }
}

/// wifi 连接成功要做的一些内容
#[allow(clippy::arc_with_non_send_sync)]
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
                log::error!("sync_firmware failed: {e}");
            }
        }
        Err(e) => log::warn!("create ota failed, {e:?}"),
    }
    Ok(())
}
