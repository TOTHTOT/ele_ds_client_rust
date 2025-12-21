use ele_ds_client_rust::{
    cmd_menu::{ShellInterface, ROOT_MENU},
    communication::{ele_ds_http_client, ota},
};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::wifi;
use esp_idf_svc::wifi::{AuthMethod, EspWifi};
use menu::Runner;
use std::io::{self, Read, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("system start, build info: {} 12", env!("BUILD_TIME"));

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspNvsPartition::<NvsDefault>::take()?;
    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs.clone()))?;
    wifi_connect(&mut wifi, "esp-2.4G", "12345678..")?;
    let client = Arc::new(Mutex::new(ele_ds_http_client::EleDsHttpClient::new(
        "https://60.215.128.73:12675",
    )?));
    let client_ota = client.clone();
    let ota = ota::Ota::new(client_ota)?;
    if let Err(e) = ota.sync_firmware() {
        log::error!("sync_firmware failed: {}", e);
    }

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
    }
}

pub fn wifi_connect(wifi: &mut EspWifi, ssid: &str, password: &str) -> anyhow::Result<()> {
    let ssid = heapless::String::<32>::from_str(ssid)
        .map_err(|_| anyhow::anyhow!("ssid too long:{ssid}"))?;
    let password = heapless::String::<64>::from_str(password)
        .map_err(|_| anyhow::anyhow!("passwd too long:{password}"))?;
    let wifi_cfg = wifi::Configuration::Client(wifi::ClientConfiguration {
        ssid,
        password,
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    });
    wifi.set_configuration(&wifi_cfg)?;
    wifi.start()?;
    wifi.connect()?;

    for i in 1..=30 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if wifi.is_connected()? {
            let netif = wifi.sta_netif();
            if let Ok(ip_info) = netif.get_ip_info() {
                if !ip_info.ip.is_unspecified() {
                    log::info!("WiFi connected IP: {:?}, total used time: {i}", ip_info.ip);
                    return Ok(());
                }
            }
        }
    }
    anyhow::bail!("WiFi connect failed");
}
