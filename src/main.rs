use ele_ds_client_rust::{ele_ds_http_client, ota};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::wifi;
use esp_idf_svc::wifi::{AuthMethod, EspWifi};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("system start, build info: {}", env!("BUILD_TIME"));

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspNvsPartition::<NvsDefault>::take()?;
    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs.clone()))?;
    wifi_connect(&mut wifi, "esp-2.4G", "12345678..")?;
    let client = Arc::new(Mutex::new(ele_ds_http_client::EleDsHttpClient::new(
        "http://192.168.137.1:24680",
    )?));
    let client_ota = client.clone();
    let ota = ota::Ota::new(client_ota)?;
    ota.get_upgrade_file("firmware.bin")?;
    /*let is_need_upgrade = ota.is_need_upgrade()?;
    log::info!("is need ota, {is_need_upgrade}");
    if is_need_upgrade {
        // client
    }*/
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
