mod psram;

use crate::communication::http_server::HttpServer;
use crate::device_config::DeviceConfig;
use crate::file_system::nvs_flash_filesystem_init;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use embedded_svc::wifi;
use embedded_svc::wifi::AuthMethod;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi;
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::wifi::EspWifi;
use ssd1680::color::{Black, Red};
use ssd1680::prelude::{Display, DisplayAnyIn, DisplayRotation, Ssd1680};
use std::str::FromStr;

#[allow(dead_code)]
pub struct BoardPeripherals<'d> {
    wifi: EspWifi<'d>,
    http_server: HttpServer<'d>,
    device_config: DeviceConfig,
}
impl<'d> BoardPeripherals<'d> {
    pub fn new() -> anyhow::Result<BoardPeripherals<'d>> {
        let peripherals = Peripherals::take()?;
        let sysloop = EspSystemEventLoop::take()?;
        let nvs = EspNvsPartition::<NvsDefault>::take()?;
        psram::check_psram();

        let device_config = BoardPeripherals::init_filesystem_load_config()?;

        let spi = peripherals.spi2;

        let sclk = peripherals.pins.gpio4;
        let sdo = peripherals.pins.gpio5;
        let rst = PinDriver::output(peripherals.pins.gpio6)?;
        let dc = PinDriver::output(peripherals.pins.gpio7)?;
        let cs = peripherals.pins.gpio15;
        let busy = PinDriver::input(peripherals.pins.gpio16)?;

        let spi = SpiDriver::new(
            spi,
            sclk,
            sdo,
            None::<AnyIOPin>,
            &SpiDriverConfig::default(),
        )?;

        let mut spi = SpiDeviceDriver::new(spi, Some(cs), &spi::config::Config::new())?;

        let mut delay = Ets;
        let mut ssd1680 = Ssd1680::new(&mut spi, busy, dc, rst, &mut delay, 128, 296).unwrap();
        ssd1680.clear_bw_frame().unwrap();
        let mut display_bw = DisplayAnyIn::bw(128, 296);
        display_bw.set_rotation(DisplayRotation::Rotate270);
        Rectangle::new(Point::new(0, 20), Size::new(40, 40))
            .into_styled(PrimitiveStyle::with_fill(Black))
            .draw(&mut display_bw)
            .unwrap();

        Circle::new(Point::new(80, 80), 40)
            .into_styled(PrimitiveStyle::with_fill(Black))
            .draw(&mut display_bw)
            .unwrap();
        log::info!("Send bw frame to display");
        ssd1680.update_bw_frame(display_bw.buffer()).unwrap();
        ssd1680.display_frame(&mut delay).unwrap();

        let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs.clone()))?;
        if let Some(ssid) = device_config.wifi_ssid.as_ref().take() {
            Self::wifi_connect(
                &mut wifi,
                ssid,
                device_config
                    .wifi_password
                    .as_ref()
                    .take()
                    .expect("get wifi_password failed"),
            )?;
        }
        let http_server = HttpServer::new()?;
        Ok(BoardPeripherals {
            wifi,
            http_server,
            device_config,
        })
    }

    fn init_filesystem_load_config() -> anyhow::Result<DeviceConfig> {
        nvs_flash_filesystem_init()?;
        let device_config = DeviceConfig::load_config()?;
        log::info!("device config: {:?}", device_config);
        Ok(device_config)
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
}
