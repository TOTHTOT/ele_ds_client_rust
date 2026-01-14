use ele_ds_client_rust::board::button::KeyClickedType;
use ele_ds_client_rust::board::peripheral::ActivePage;
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
    let screen = board.screen.clone();
    let screen_exit = board.screen_exit.clone();
    let key_exit = board.key_read_exit.clone();

    let (key_tx, screen_rx) = std::sync::mpsc::channel();
    // 屏幕刷新线程
    let _ = std::thread::Builder::new()
        .stack_size(1024 * 10)
        .name(String::from("epd"))
        .spawn(move || {
            while !screen_exit.load(std::sync::atomic::Ordering::Relaxed) {
                let cur_set_page = screen_rx.recv().unwrap();
                let screen = screen.clone();
                log::info!("cur_set page: {cur_set_page:?}");
                if let Err(e) = mouse_food_test(screen, cur_set_page) {
                    log::warn!("refresh screen failed: {e:?}");
                };
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

    // 按键命令接收线程
    let key_rx = board.key_rx.take().expect("key rx, take filed");
    std::thread::spawn(move || {
        while !key_exit.load(std::sync::atomic::Ordering::Relaxed) {
            let key_info = key_rx.recv().unwrap();
            if key_info.click_type == KeyClickedType::SingleClicked {
                if let Ok(cur_set_page) = ActivePage::try_from(key_info.idx) {
                    key_tx.send(cur_set_page).unwrap();
                }
            }
            log::info!("{key_info:?}");
        }
    });

    let board = Arc::new(Mutex::new(board));

    let _http_server = HttpServer::new()?;

    loop {
        {
            let mut board = board
                .lock()
                .map_err(|e| anyhow::anyhow!("lock board failed: {e:?}"))?;
            let device_status = board.read_all_sensor()?;
            log::info!("device_status: {device_status:?}");
        }
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
