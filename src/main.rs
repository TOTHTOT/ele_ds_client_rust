use chrono::Timelike;
use ele_ds_client_rust::board::button::KeyClickedType;
use ele_ds_client_rust::board::power_manage::next_minute_left_time;
use ele_ds_client_rust::board::{get_clock_ntp, psram};
use ele_ds_client_rust::communication::http_server::HttpServer;
use ele_ds_client_rust::communication::weather::Weather;
use ele_ds_client_rust::device_config::DeviceConfig;
use ele_ds_client_rust::ui::popup::PopupMsg;
use ele_ds_client_rust::ui::{mouse_food_test, ScreenEvent};
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

    let mut device_config = BoardPeripherals::init_filesystem_load_config()?;
    get_clock_ntp::set_time_zone(device_config.time_zone.as_str())?;
    device_config.boot_times_add()?;
    let power_on_ui_page = device_config.current_page;
    let device_config = Arc::new(Mutex::new(device_config));
    let device_config_ui = device_config.clone();

    let mut screen = board.screen.take().expect("no screen");
    // 赋值屏幕默认传感器数据
    let sensor_data = board.read_all_sensor()?;
    screen.last_sensor_status = Some(sensor_data);

    let screen_exit = board.screen_exit.clone();
    let key_exit = board.key_read_exit.clone();

    let (screen_tx, screen_rx) = std::sync::mpsc::channel();
    let screen_tx_main = screen_tx.clone();
    // 上电同步掉电时的页面, 避免保存的页面和实际不一样
    screen_tx_main.send(ScreenEvent::Refresh(power_on_ui_page))?;
    // 屏幕刷新线程
    let _ = std::thread::Builder::new()
        .stack_size(1024 * 10)
        .name(String::from("epd"))
        .spawn(move || {
            while !screen_exit.load(std::sync::atomic::Ordering::Relaxed) {
                let Ok(event) = screen_rx.recv() else {
                    continue;
                };
                match event {
                    ScreenEvent::Refresh(cur_set_page) => {
                        log::info!("cur_set page: {cur_set_page:?}");
                        if let Err(e) = mouse_food_test(
                            &mut screen,
                            device_config_ui.clone(),
                            cur_set_page,
                            None,
                        ) {
                            log::warn!("refresh screen failed: {e:?}");
                        };
                    }
                    ScreenEvent::UpdateSensorsData(sensors_data) => {
                        screen.last_sensor_status = Some(sensors_data);
                    }
                    ScreenEvent::Popup(msg) => {
                        let cur_page = screen.current_page;
                        if let Err(e) = mouse_food_test(
                            &mut screen,
                            device_config_ui.clone(),
                            cur_page,
                            Some(msg),
                        ) {
                            log::warn!("show popup screen failed: {e:?}");
                        };
                    }
                }
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
                    if let Err(e) = screen_tx.send(ScreenEvent::Refresh(cur_set_page)) {
                        log::warn!("refresh active_page failed: {e:?}");
                    }
                }
            }
            if key_info.click_type == KeyClickedType::DoubleClicked {
                if let Err(e) = screen_tx.send(ScreenEvent::Popup(PopupMsg::new(
                    "Warning".to_string(),
                    "test".to_string(),
                ))) {
                    log::warn!("Popup failed: {e:?}");
                }
            }
            log::info!("{key_info:?}");
        }
    });
    if let Err(e) = connect_net(&mut board, device_config.clone()) {
        log::warn!("connect_net failed: {e:?}");
    }
    let board = Arc::new(Mutex::new(board));

    let _http_server = HttpServer::new()?;
    let mut loop_times = 0; // 不断电情况下的循环次数, 可以控制一些第一次循环不执行的功能
    loop {
        let mut sleep_time = u64::MAX;
        let mut board = board
            .lock()
            .map_err(|e| anyhow::anyhow!("lock board failed: {e:?}"))?;
        let sensors_data = board.read_all_sensor()?;
        screen_tx_main.send(ScreenEvent::UpdateSensorsData(sensors_data))?;
        log::info!("last sensor_status: {:?}", sensors_data);

        /* 界面更新区分两种情况:
            1. 如果一直在运行状态时每分钟更新时间, 这时要发信号
            2. 如果是从深度睡眠唤醒, 这时就不要再发信号了, 但是还没做
        */
        if let Ok(config) = device_config.lock() {
            // 每分钟更新屏幕, 实际刷不刷新屏幕由屏幕自己决定
            if loop_times > 1 {
                screen_tx_main.send(ScreenEvent::Refresh(config.current_page))?;
            }
            // 如果当前是主页面或者传感器页面就定时刷新数据, 不然的话就睡眠最大时间
            if config.current_page.cur_set_page_is_need_refresh() {
                sleep_time = next_minute_left_time();
            }
            config.save_config()?;
        };
        drop(board);

        loop_times += 1;
        psram::check_psram();
        std::thread::sleep(std::time::Duration::from_micros(sleep_time));
        // ele_ds_client_rust::board::power_manage::enter_deep_sleep_mode_per_minute();
    }
}

/// 连接网络
fn connect_net(
    board: &mut BoardPeripherals,
    device_config: Arc<Mutex<DeviceConfig>>,
) -> anyhow::Result<()> {
    let Ok(mut device_config) = device_config.lock() else {
        anyhow::bail!("lock failed");
    };
    if !device_config.is_need_connect_wifi() {
        return Ok(());
    }
    if BoardPeripherals::wifi_connect(
        &mut board.wifi,
        &device_config.wifi_ssid.clone(),
        &device_config.wifi_password.clone(),
        device_config.wifi_max_link_time,
    )
    .is_ok()
    {
        if let Err(e) = after_wifi_established() {
            log::warn!("after_wifi_established failed: {e:?}");
        }
        if DeviceConfig::current_time_is_too_old() {
            if let Err(e) = get_clock_ntp::set_ntp_time(
                device_config.wifi_max_link_time / 2,
                device_config.time_zone.as_str(),
            ) {
                log::warn!("failed to set NTP time: {e:?}");
            }
        }
        if let Err(e) = update_weather_per_hour(&mut device_config) {
            log::warn!("update_weather_per_hour failed: {e:?}");
        }
    }
    Ok(())
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
/// 每小时更新一次时间, 默认都返回 default_data , 除非 get_ui_need_data()失败
fn update_weather_per_hour(config: &mut DeviceConfig) -> anyhow::Result<()> {
    let now = chrono::Local::now().hour();
    if config.last_update_weather != now {
        config.weather = Option::from(
            Weather::new(&config.city_name, &config.weather_api_key).get_weather_hefeng()?,
        );
        config.last_update_weather = now;
        Ok(())
    } else {
        Ok(())
    }
}
