use ele_ds_client_rust::board::button::KeyClickedType;
use ele_ds_client_rust::board::power_manage::next_minute_left_time;
use ele_ds_client_rust::board::psram;
use ele_ds_client_rust::communication::http_server::HttpServer;
use ele_ds_client_rust::ui::mouse_food_test;
use ele_ds_client_rust::{
    board::peripheral::BoardPeripherals,
    communication::{http_client, ota},
    ActivePage,
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
    {
        let sensor_data = board.read_all_sensor()?;
        let mut screen = board.screen.lock().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        screen.last_sensor_status = Some(sensor_data);
    }
    let screen = board.screen.clone();
    let screen_main = board.screen.clone();
    let screen_exit = board.screen_exit.clone();
    let key_exit = board.key_read_exit.clone();

    let (screen_tx, screen_rx) = std::sync::mpsc::channel();
    let screen_tx_main = screen_tx.clone();
    // 上电同步掉电时的页面, 避免保存的页面和实际不一样
    screen_tx_main.send(board.device_config.current_page)?;
    // 屏幕刷新线程
    let _ = std::thread::Builder::new()
        .stack_size(1024 * 10)
        .name(String::from("epd"))
        .spawn(move || {
            while !screen_exit.load(std::sync::atomic::Ordering::Relaxed) {
                let Ok(cur_set_page) = screen_rx.recv() else {
                    continue;
                };
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
            let Ok(key_info) = key_rx.recv() else {
                log::warn!("key receive failed");
                continue;
            };
            if key_info.click_type == KeyClickedType::SingleClicked {
                if let Ok(cur_set_page) = ActivePage::try_from(key_info.idx) {
                    if let Err(e) = screen_tx.send(cur_set_page) {
                        log::warn!("refresh active_page failed: {e:?}");
                    }
                }
            }
            log::info!("{key_info:?}");
        }
    });

    let board = Arc::new(Mutex::new(board));

    let _http_server = HttpServer::new()?;
    let mut loop_times = 0; // 不断电情况下的循环次数, 可以控制一些第一次循环不执行的功能
    loop {
        let mut sleep_time = u64::MAX;
        let mut board = board
            .lock()
            .map_err(|e| anyhow::anyhow!("lock board failed: {e:?}"))?;
        let mut screen = screen_main
            .lock()
            .map_err(|e| anyhow::anyhow!("lock board failed: {e:?}"))?;
        screen.last_sensor_status = Some(board.read_all_sensor()?);
        log::info!("last sensor_status: {:?}", screen.last_sensor_status);

        /* 界面更新区分两种情况:
            1. 如果一直在运行状态时每分钟更新时间, 这时要发信号
            2. 如果是从深度睡眠唤醒, 这时就不要再发信号了, 但是还没做
        */
        if screen.current_page.cur_set_page_is_need_refresh() && loop_times > 1 {
            screen_tx_main.send(screen.current_page)?;
        }
        if screen.current_page == ActivePage::Home || screen.current_page == ActivePage::Sensor {
            // 如果当前是主页面或者传感器页面就定时刷新数据, 不然的话就睡眠最大时间
            sleep_time = next_minute_left_time();
        }
        if board.device_config.current_page != screen.current_page {
            board.device_config.current_page = screen.current_page;
            board.device_config.save_config()?;
        }
        drop(screen);
        drop(board);

        loop_times += 1;
        psram::check_psram();
        std::thread::sleep(std::time::Duration::from_micros(sleep_time));
        // ele_ds_client_rust::board::power_manage::enter_deep_sleep_mode_per_minute();
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
