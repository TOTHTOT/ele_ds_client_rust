use crate::device_config::DeviceConfig;
use crate::file_system::nvs_flash_filesystem_init;
use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::{geometry, Drawable};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_svc::wifi;
use embedded_svc::wifi::AuthMethod;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{Gpio0, Gpio2, Gpio3, Gpio4, Gpio5, Pin};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::{SpiBusDriver, SpiDriver};
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::wifi::EspWifi;
use ssd1675::{Builder, Dimensions, Display, GraphicDisplay, Rotation};
use std::str::FromStr;

pub struct BoardPeripherals<'d> {
    wifi: EspWifi<'d>,
}
impl<'d> BoardPeripherals<'d> {
    pub fn new() -> anyhow::Result<BoardPeripherals<'d>> {
        let peripherals = Peripherals::take()?;
        let sysloop = EspSystemEventLoop::take()?;
        let nvs = EspNvsPartition::<NvsDefault>::take()?;
        BoardPeripherals::init_filesystem_load_config()?;

        let driver_config = Default::default();
        let spi_drv = SpiDriver::new(
            peripherals.spi2,
            peripherals.pins.gpio12,
            peripherals.pins.gpio11,
            None::<Gpio0>,
            &driver_config,
        )?;

        let wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs.clone()))?;
        Ok(BoardPeripherals { wifi })
    }

    fn init_filesystem_load_config() -> anyhow::Result<()> {
        nvs_flash_filesystem_init()?;
        let device_config = DeviceConfig::load_config()?;
        log::info!("device config: {:?}", device_config);
        Ok(())
    }
    fn init_epd_display<DC, CS>(
        spi: SpiBusDriver<'d, SpiDriver<'d>>,
        cs: CS,
        dc: DC,
    ) -> anyhow::Result<()>
    where
        CS: embedded_hal::digital::OutputPin,
        DC: embedded_hal::digital::OutputPin,
    {
        let controller = ssd1675::Interface::new(spi, cs, busy, dc, reset);
        let config = Builder::new()
            .dimensions(Dimensions {
                rows: 264,
                cols: 176,
            })
            .rotation(Rotation::Rotate270)
            .lut(&LUT)
            .build()
            .expect("invalid configuration");

        let display = Display::new(controller, config);
        let mut display = GraphicDisplay::new(display, &mut black_buffer, &mut red_buffer);

        let image_raw: ImageRawLE<Rgb565> =
            ImageRaw::new(include_bytes!("../../assets/ferris.raw"), 86);
        let image = Image::new(&image_raw, geometry::Point::new(26, 8));
        image.draw(&mut display).unwrap();
        Ok(())
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
