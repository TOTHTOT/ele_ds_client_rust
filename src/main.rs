use ele_ds_client_rust::{
    board::BoardPeripherals,
    cmd_menu::{ShellInterface, ROOT_MENU},
    communication::{http_client, ota},
};
use menu::Runner;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
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

    let mut stdin = io::stdin();
    std::thread::spawn(move || {
        let mut buffer = [0u8; 128];
        let mut context = ();
        let mut runner = Runner::new(ROOT_MENU, &mut buffer, ShellInterface, &mut context);
        log::info!("shell start");
        println!("\nESP32 Shell Tool Ready (WDT disabled for this thread)");
        print!("> ");
        let _ = io::stdout().flush().unwrap();

        loop {
            let mut byte = [0u8; 1];
            match stdin.read(&mut byte) {
                Ok(n) if n > 0 => {
                    let c = byte[0];
                    print!("{}", c as char);
                    let _ = io::stdout().flush();

                    runner.input_byte(c, &mut context);

                    if c == b'\r' || c == b'\n' {
                        // println!("");
                        print!("> ");
                        let _ = io::stdout().flush();
                    }
                }
                Ok(_) | Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            }
        }
    });

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
