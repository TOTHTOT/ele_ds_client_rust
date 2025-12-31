mod psram;

use crate::communication::http_server::HttpServer;
use crate::device_config::DeviceConfig;
use crate::file_system::nvs_flash_filesystem_init;
use display_interface_spi::SPIInterface;
// use embedded_graphics::geometry::Point;
// use embedded_graphics::mono_font::MonoTextStyle;
// use embedded_graphics::text::{Text, TextStyle};
// use embedded_graphics::Drawable;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_svc::wifi;
use embedded_svc::wifi::AuthMethod;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::{SpiBusDriver, SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::wifi::EspWifi;
use profont::PROFONT_24_POINT;
use std::str::FromStr;
use weact_studio_epd::graphics::{Display290BlackWhite, DisplayRotation};
use weact_studio_epd::{Color, WeActStudio290BlackWhiteDriver};

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

        let rst = PinDriver::output(peripherals.pins.gpio6)?;
        let dc = PinDriver::output(peripherals.pins.gpio7)?;
        let busy = PinDriver::input(peripherals.pins.gpio16)?;
        let delay = Ets;

        let sclk = peripherals.pins.gpio4;
        let sdo = peripherals.pins.gpio5;
        let cs = PinDriver::output(peripherals.pins.gpio15)?;

        // let spi_bus = SpiBusDriver::new()
        // let spi = SpiDriver::new(
        //     peripherals.spi2,
        //     sclk,
        //     sdo,
        //     None::<AnyIOPin>,
        //     &SpiDriverConfig::default(),
        // )?;
        // let spi = SpiDeviceDriver::new(
        //     spi,
        //     None, &hal::spi::config::Config::new(),
        // )?;
        // log::info!("Initializing SPI Device...");
        // let spi_device = ExclusiveDevice::new(spi, cs, delay).expect("SPI device initialize error");
        // let spi_interface = SPIInterface::new(spi_device, dc);
        // let spi_bus = Spi
        // Setup EPD
        log::info!("Intializing EPD...");
        // let mut driver = WeActStudio290BlackWhiteDriver::new(spi_interface, busy, rst, Ets);
        // let mut display = Display290BlackWhite::new();
        // display.set_rotation(DisplayRotation::Rotate90);
        // driver.init().unwrap();
        //
        // let style = MonoTextStyle::new(&PROFONT_24_POINT, Color::Black);
        // let _ = Text::with_text_style(
        //     "Hello World!",
        //     Point::new(8, 68),
        //     style,
        //     TextStyle::default(),
        // )
        // .draw(&mut display);
        //
        // driver.full_update(&display).unwrap();
        //
        // log::info!("Sleeping for 5s...");
        // driver.sleep().unwrap();
        // std::thread::sleep(std::time::Duration::from_secs(5));

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
