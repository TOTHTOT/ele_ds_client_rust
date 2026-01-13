use crate::board::button::DeviceButton;
use crate::board::es8388::driver::{Es8388, RunMode};
use crate::board::power_manage::DeviceBattery;
use crate::board::share_i2c_bus::SharedI2cDevice;
use crate::board::{es8388, get_clock_ntp, psram};
use crate::device_config::DeviceConfig;
use crate::file_system::nvs_flash_filesystem_init;
use anyhow::Context;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use embedded_sht3x::{Measurement, Repeatability, Sht3x, DEFAULT_I2C_ADDRESS};
use embedded_svc::wifi;
use embedded_svc::wifi::AuthMethod;
use enumset::EnumSet;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{AnyIOPin, AnyInputPin, IOPin, Input, Output, PinDriver};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::i2s::I2sDriver;
use esp_idf_svc::hal::interrupt::InterruptType;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::Hertz;
use esp_idf_svc::hal::spi;
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::wifi::EspWifi;
use ssd1680::color::Black;
use ssd1680::prelude::{Display, DisplayAnyIn, DisplayRotation, Ssd1680};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

type Ssd1680DisplayType = Ssd1680<
    SpiDeviceDriver<'static, SpiDriver<'static>>,
    PinDriver<'static, AnyIOPin, Input>,  // BUSY
    PinDriver<'static, AnyIOPin, Output>, // DC
    PinDriver<'static, AnyIOPin, Output>, // RST
>;
type Es8388Type = Es8388<
    'static,
    SharedI2cDevice<Arc<Mutex<I2cDriver<'static>>>>,
    PinDriver<'static, AnyIOPin, Output>,
>;
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct DeviceStatus {
    sht3x_measure: Measurement,
}
#[allow(dead_code)]
pub struct BoardPeripherals {
    wifi: EspWifi<'static>,
    pub device_config: DeviceConfig,
    pub bw_buf: DisplayAnyIn,
    pub delay: Ets,
    pub ssd1680: Ssd1680DisplayType,
    pub es8388: Arc<Mutex<Es8388Type>>,
    vout_3v3: PinDriver<'static, AnyIOPin, Output>,
    sht3x_rst: PinDriver<'static, AnyIOPin, Output>,
    pub sht3x: Sht3x<SharedI2cDevice<Arc<Mutex<I2cDriver<'static>>>>, Ets>,
    pub device_battery: DeviceBattery<
        PinDriver<'static, AnyIOPin, Input>,
        PinDriver<'static, AnyIOPin, Input>,
        PinDriver<'static, AnyIOPin, Input>,
    >,
    pub device_button: DeviceButton,
    pub key_read_exit: Arc<AtomicBool>, // 发送信号让读按键线程退出
}

#[allow(dead_code)]
impl BoardPeripherals {
    pub fn new() -> anyhow::Result<BoardPeripherals> {
        let peripherals = Peripherals::take()?;
        let sysloop = EspSystemEventLoop::take()?;
        let nvs = EspNvsPartition::<NvsDefault>::take()?;
        psram::check_psram();

        let device_config = BoardPeripherals::init_filesystem_load_config()?;
        get_clock_ntp::set_time_zone(device_config.time_zone.as_str())?;

        // 基本io口初始化
        let mut vout_3v3 = PinDriver::output(peripherals.pins.gpio10.downgrade())?;
        vout_3v3.set_high()?;
        let mut sht3x_rst = PinDriver::output(peripherals.pins.gpio19.downgrade())?;
        sht3x_rst.set_high()?;
        let mut device_battery = DeviceBattery::new(
            PinDriver::input(peripherals.pins.gpio12.downgrade())?,
            PinDriver::input(peripherals.pins.gpio13.downgrade())?,
            PinDriver::input(peripherals.pins.gpio14.downgrade())?,
        );
        log::info!("current battery: {:?}", device_battery.current_vbat()?);

        let i2c_config = I2cConfig {
            baudrate: Hertz(400_000),
            sda_pullup_enabled: true,
            scl_pullup_enabled: true,
            timeout: None,
            intr_flags: EnumSet::<InterruptType>::empty(),
        };
        let mut i2c_driver = I2cDriver::new(
            peripherals.i2c0,
            peripherals.pins.gpio8,
            peripherals.pins.gpio18,
            &i2c_config,
        )?;

        // 可能是底层库有bug, 必须先执行一遍i2c的读写操作才能进行后续读传感器, 不然会报错 ESP_ERR_TIMEOUT
        Self::i2c_scan(&mut i2c_driver);
        let iic_bus = Arc::new(Mutex::new(i2c_driver));
        let mut sht3x = Sht3x::new(SharedI2cDevice(iic_bus.clone()), DEFAULT_I2C_ADDRESS, Ets);
        sht3x.repeatability = Repeatability::High;

        // 按键初始化
        let (key_tx, _ket_rx) = std::sync::mpsc::channel();
        let key_pins: Vec<AnyInputPin> = vec![
            peripherals.pins.gpio3.into(),
            peripherals.pins.gpio46.into(),
            peripherals.pins.gpio9.into(),
        ];
        let key_read_exit = Arc::new(AtomicBool::new(false));
        let key_read_exit_clone = key_read_exit.clone();
        let device_button = DeviceButton::new(key_pins, key_tx, key_read_exit_clone)?;

        let spi = peripherals.spi2;
        let sclk = peripherals.pins.gpio4;
        let sdo = peripherals.pins.gpio5;
        let rst = PinDriver::output(peripherals.pins.gpio6.downgrade())?;
        let dc = PinDriver::output(peripherals.pins.gpio7.downgrade())?;
        let cs = peripherals.pins.gpio15;
        let busy = PinDriver::input(peripherals.pins.gpio16.downgrade())?;

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

        let i2s = peripherals.i2s0;
        // i2s相关初始化
        let i2s_driver = I2sDriver::new_std_bidir(
            i2s,
            &es8388::driver::default_i2s_config(),
            peripherals.pins.gpio47,      // bclk i2s总线的时钟
            peripherals.pins.gpio45,      // din codec支持录音功能可以把麦克风数据回传给单片机
            peripherals.pins.gpio1,       // dout 音频输出
            Some(peripherals.pins.gpio2), // mclk 给codec芯片提供的始终
            peripherals.pins.gpio48,      // ws 左右声道选择
        )
        .context("Failed to initialize I2S bidirectional driver")?;
        let es8388_i2c = SharedI2cDevice(iic_bus.clone());
        let en_spk = PinDriver::output(peripherals.pins.gpio20.downgrade())?;
        let mut es8388 = Es8388::new(
            i2s_driver,
            es8388_i2c,
            en_spk,
            es8388::driver::CHIP_ADDR,
            RunMode::AdcDac,
        );
        es8388.init()?;
        es8388.start()?;
        es8388.set_speaker(true)?;
        let regs = es8388.read_all()?;
        log::info!("es8388 regs: {:?}", &regs);
        let es8388 = Arc::new(Mutex::new(es8388));
        let es8388_clone = es8388.clone();
        std::thread::spawn(move || loop {
            let mut es8388 = es8388_clone.lock().unwrap();
            let buf = Box::new([100_u8; 1024]);
            // if let Err(e) = es8388.read_audio(&mut *buf, 1000) {
            //     log::error!("Failed to read audio buffer: {e:?}");
            // }
            // log::info!("es8388 = {buf:?}");
            if let Err(e) = es8388.write_audio(&*buf, 1000) {
                log::error!("Failed to write audio buffer: {e:?}");
            }
            log::info!("es8388: {:?}", &buf);
            std::thread::sleep(std::time::Duration::from_millis(5000));
        });
        let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs.clone()))?;
        if let Err(e) = Self::wifi_connect(&mut wifi, &device_config) {
            log::warn!("Wifi connect error: {e:?}");
        }

        Ok(BoardPeripherals {
            wifi,
            device_config,
            bw_buf: display_bw,
            delay,
            es8388,
            ssd1680,
            vout_3v3,
            sht3x_rst,
            sht3x,
            device_battery,
            device_button,
            key_read_exit,
        })
    }

    /// 一次性读取所有传感器数据接口, 保存在 DeviceStatus
    pub fn read_all_sensor(&mut self) -> anyhow::Result<DeviceStatus> {
        let sht3x_measure = self
            .sht3x
            .single_measurement()
            .map_err(|e| anyhow::anyhow!("sht3x get data failed: {e:?}"))?;
        Ok(DeviceStatus { sht3x_measure })
    }

    /// 测试功能, 检查总线上的i2c设备
    pub fn i2c_scan(i2c: &mut I2cDriver) {
        log::info!("Scanning I2C bus...");
        for addr in 1..127 {
            if i2c.write(addr, &[], 50).is_ok() {
                log::info!("Found device at address: 0x{addr:02X}");
            }
        }
        log::info!("Scan complete.");
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

impl Drop for BoardPeripherals {
    fn drop(&mut self) {
        log::warn!("Dropping BoardPeripherals, close power");
        self.vout_3v3.set_low().unwrap();
    }
}
