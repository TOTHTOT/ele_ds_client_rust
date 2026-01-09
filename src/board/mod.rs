pub mod es8388;
pub mod get_clock_ntp;
mod psram;

use crate::board::es8388::driver::{Es8388, RunMode};
use crate::board::share_i2c_bus::SharedI2cDevice;
use crate::communication::http_server::HttpServer;
use crate::device_config::DeviceConfig;
use crate::file_system::nvs_flash_filesystem_init;
use anyhow::Context;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use embedded_sht3x::{Sht3x, DEFAULT_I2C_ADDRESS};
use embedded_svc::wifi;
use embedded_svc::wifi::AuthMethod;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{AnyIOPin, Gpio16, Gpio20, Gpio6, Gpio7, Input, Output, PinDriver};
use esp_idf_svc::hal::i2c;
use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::hal::i2s::I2sDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi;
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::wifi::EspWifi;
use ssd1680::color::Black;
use ssd1680::prelude::{Display, DisplayAnyIn, DisplayRotation, Ssd1680};
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
// use shared_bus::I2cProxy;

// 如果是单线程操作
type Ssd1680DisplayType<'d> = Ssd1680<
    SpiDeviceDriver<'d, SpiDriver<'d>>,
    PinDriver<'d, Gpio16, Input>, // BUSY
    PinDriver<'d, Gpio7, Output>, // DC
    PinDriver<'d, Gpio6, Output>, // RST
>;
type Es8388Type<'d> = Es8388<'d, SharedI2cDevice<I2cDriver<'d>>, PinDriver<'d, Gpio20, Output>>;
// type Es8388Type<'d> = Es8388<'d, I2cProxy<'d, I2cDriver<'d>>, PinDriver<'d, Gpio20, Output>>;

#[allow(dead_code)]
pub struct BoardPeripherals<'d> {
    wifi: EspWifi<'d>,
    http_server: HttpServer<'d>,
    // iic_bus: RefCell<I2cDriver<'d>>,
    pub device_config: DeviceConfig,
    pub bw_buf: DisplayAnyIn,
    pub ssd1680: Ssd1680DisplayType<'d>,
    pub delay: Ets,
    pub es8388: Es8388Type<'d>,
}
#[allow(dead_code)]
impl<'d> BoardPeripherals<'d> {
    pub fn new() -> anyhow::Result<BoardPeripherals<'d>> {
        let peripherals = Peripherals::take()?;
        let sysloop = EspSystemEventLoop::take()?;
        let nvs = EspNvsPartition::<NvsDefault>::take()?;
        psram::check_psram();

        let device_config = BoardPeripherals::init_filesystem_load_config()?;
        get_clock_ntp::set_time_zone(device_config.time_zone.as_str())?;

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
        let spi = SpiDeviceDriver::new(spi, Some(cs), &spi::config::Config::new())?;

        let mut delay = Ets;
        let ssd1680 = Ssd1680::new(spi, busy, dc, rst, &mut delay, 128, 296).unwrap();
        let mut display_bw = DisplayAnyIn::bw(128, 296);
        display_bw.set_rotation(DisplayRotation::Rotate270);
        let i2c_driver = I2cDriver::new(
            peripherals.i2c0,
            peripherals.pins.gpio8,
            peripherals.pins.gpio18,
            &i2c::I2cConfig::default(),
        )?;
        let iic_bus = Rc::new(RefCell::new(i2c_driver));
        let _sensor = Sht3x::new(SharedI2cDevice(iic_bus.clone()), DEFAULT_I2C_ADDRESS, Ets);

        let i2s = peripherals.i2s0;

        // i2s相关初始化
        let i2s_driver = I2sDriver::new_std_bidir(
            i2s,
            &es8388::driver::default_i2s_config(),
            peripherals.pins.gpio47,
            peripherals.pins.gpio45,
            peripherals.pins.gpio1,
            Some(peripherals.pins.gpio2),
            peripherals.pins.gpio48,
        )
        .context("Failed to initialize I2S bidirectional driver")?;
        let es8388_i2c = SharedI2cDevice(iic_bus.clone());
        let en_spk = PinDriver::output(peripherals.pins.gpio20)?;
        let es8388 = Es8388::new(
            i2s_driver,
            es8388_i2c,
            // i2c_ref_cell.acquire_i2c(),
            en_spk,
            es8388::driver::CHIP_ADDR,
            RunMode::AdcDac,
        );
        let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs.clone()))?;
        if let Err(e) = Self::wifi_connect(&mut wifi, &device_config) {
            log::warn!("Wifi connect error: {e:?}");
        }
        let http_server = HttpServer::new()?;

        Ok(BoardPeripherals {
            wifi,
            http_server,
            device_config,
            bw_buf: display_bw,
            ssd1680,
            delay,
            es8388,
        })
    }

    fn init_filesystem_load_config() -> anyhow::Result<DeviceConfig> {
        nvs_flash_filesystem_init()?;
        let device_config = DeviceConfig::load_config()?;
        log::info!("device config: {device_config:?}");
        Ok(device_config)
    }
    pub fn wifi_connect(wifi: &mut EspWifi, config: &DeviceConfig) -> anyhow::Result<()> {
        if !config.is_need_connect_wifi() && !DeviceConfig::current_time_is_too_old() {
            log::info!(
                "Wifi config is not need connect, boot_times: {}",
                config.boot_times
            );
            return Ok(());
        }
        let ssid_str = config
            .wifi_ssid
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("get wifi name failed"))?;
        let ssid = heapless::String::<32>::from_str(ssid_str)
            .map_err(|_| anyhow::anyhow!("ssid too long:{ssid_str}"))?;

        let password_str = config
            .wifi_password
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("get wifi name failed"))?;
        let password = heapless::String::<64>::from_str(password_str)
            .map_err(|_| anyhow::anyhow!("passwd too long:{password_str}"))?;

        let wifi_cfg = wifi::Configuration::Client(wifi::ClientConfiguration {
            ssid,
            password,
            auth_method: AuthMethod::WPA2Personal,
            ..Default::default()
        });
        wifi.set_configuration(&wifi_cfg)?;
        wifi.start()?;
        wifi.connect()?;

        for i in 1..=config.wifi_max_link_time {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if wifi.is_connected()? {
                let netif = wifi.sta_netif();
                if let Ok(ip_info) = netif.get_ip_info() {
                    if !ip_info.ip.is_unspecified() {
                        // 连接成功获取网络时间
                        if let Err(e) = get_clock_ntp::set_ntp_time(
                            config.wifi_max_link_time - i,
                            config.time_zone.as_str(),
                        ) {
                            log::warn!("failed to set NTP time: {e:?}");
                        }
                        log::info!("WiFi connected IP: {:?}, total used time: {i}", ip_info.ip);
                        return Ok(());
                    }
                }
            }
        }
        anyhow::bail!("WiFi connect failed");
    }

    pub fn test_epd_display(&mut self) -> anyhow::Result<()> {
        self.ssd1680.clear_bw_frame().unwrap();

        self.bw_buf.set_rotation(DisplayRotation::Rotate270);
        Rectangle::new(Point::new(0, 20), Size::new(40, 40))
            .into_styled(PrimitiveStyle::with_fill(Black))
            .draw(&mut self.bw_buf)
            .unwrap();

        Circle::new(Point::new(80, 80), 40)
            .into_styled(PrimitiveStyle::with_fill(Black))
            .draw(&mut self.bw_buf)
            .unwrap();
        log::info!("Send bw frame to display");
        self.ssd1680.update_bw_frame(self.bw_buf.buffer()).unwrap();
        self.ssd1680.display_frame(&mut self.delay).unwrap();
        Ok(())
    }
}

pub mod share_i2c_bus {
    use core::cell::RefCell;
    use embedded_hal::i2c::{ErrorType, I2c, Operation};
    use std::rc::Rc;

    pub struct SharedI2cDevice<T>(pub Rc<RefCell<T>>);

    impl<T: ErrorType> ErrorType for SharedI2cDevice<T> {
        type Error = T::Error;
    }

    impl<T: I2c> I2c for SharedI2cDevice<T> {
        fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
            self.0.borrow_mut().read(address, read)
        }

        fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
            self.0.borrow_mut().write(address, write)
        }

        fn write_read(
            &mut self,
            address: u8,
            write: &[u8],
            read: &mut [u8],
        ) -> Result<(), Self::Error> {
            self.0.borrow_mut().write_read(address, write, read)
        }

        fn transaction(
            &mut self,
            address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            self.0.borrow_mut().transaction(address, operations)
        }
    }
}
