use chrono::Timelike;
use esp_idf_svc::sys::*;

pub fn enter_light_sleep_mode() -> anyhow::Result<()> {
    let wakeup_time_us = 10 * 1000 * 1000;
    unsafe {
        log::info!("sleeping for {wakeup_time_us} us");
        esp_sleep_enable_timer_wakeup(wakeup_time_us);

        // 可选：设置 RTC GPIO 唤醒
        // esp_sleep_enable_ext0_wakeup(GPIO_NUM_1, 1); // 高电平唤醒
    }

    let start_us = unsafe { esp_timer_get_time() };
    // 2. 开始睡眠
    unsafe {
        esp_light_sleep_start();
    }
    let end_us = unsafe { esp_timer_get_time() };
    let actual_sleep_ms = (end_us - start_us) / 1000;

    log::info!("wakeup sleep time: {actual_sleep_ms} ms");
    Ok(())
}

/// 距离下一分钟还有多久, 返回微妙
pub fn next_minute_left_time() -> u64 {
    let now = chrono::Local::now();

    let seconds_to_wait = 59 - now.second();
    let nanos_to_wait = 1_000_000_000 - now.timestamp_nanos_opt().unwrap_or(0) % 1_000_000_000;

    (seconds_to_wait as u64 * 1_000_000) + (nanos_to_wait / 1_000) as u64
}
pub fn enter_deep_sleep_mode_per_minute() {
    let now = chrono::Local::now();
    let sleep_time_us = next_minute_left_time();

    log::info!(
        "Current time: {:02}:{:02}:{:02}, aligned sleep for {} us",
        now.hour(),
        now.minute(),
        now.second(),
        sleep_time_us
    );

    enter_deep_sleep_mode(sleep_time_us);
}
pub fn enter_deep_sleep_mode(sleep_time_us: u64) {
    unsafe {
        log::info!("sleeping for {sleep_time_us} us");
        esp_sleep_enable_timer_wakeup(sleep_time_us);

        // 可选：设置 RTC GPIO 唤醒
        // esp_sleep_enable_ext0_wakeup(GPIO_NUM_1, 1); // 高电平唤醒
    }
    unsafe {
        esp_deep_sleep_start();
    }
}

pub struct DeviceBattery<VBAT1, VBAT2, VBAT3> {
    vbat_1: VBAT1,
    vbat_2: VBAT2,
    vbat_3: VBAT3,
}
#[derive(Debug)]
pub enum DeviceBatteryType {
    PercentVbat100,
    PercentVbat100_75,
    PercentVbat75_50,
    PercentVbat50_25,
    PercentVbat25_0,
}

impl<VBAT1, VBAT2, VBAT3> DeviceBattery<VBAT1, VBAT2, VBAT3>
where
    VBAT1: embedded_hal::digital::InputPin,
    VBAT2: embedded_hal::digital::InputPin,
    VBAT3: embedded_hal::digital::InputPin,
{
    pub fn new(vbat_1: VBAT1, vbat_2: VBAT2, vbat_3: VBAT3) -> Self {
        Self {
            vbat_1,
            vbat_2,
            vbat_3,
        }
    }

    /// 获取实际电量, 根据手册需要延迟一段时间才能读完全部io信息
    pub fn current_vbat(&mut self) -> anyhow::Result<DeviceBatteryType> {
        let mut d1_active = false;
        let mut d2_active = false;
        let mut d3_active = false;

        for _ in 0..150 {
            if self.vbat_1.is_low().map_err(|e| anyhow::anyhow!("{e:?}"))? {
                d1_active = true;
            }
            if self.vbat_2.is_low().map_err(|e| anyhow::anyhow!("{e:?}"))? {
                d2_active = true;
            }
            if self.vbat_3.is_low().map_err(|e| anyhow::anyhow!("{e:?}"))? {
                d3_active = true;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if d3_active {
            Ok(DeviceBatteryType::PercentVbat100)
        } else if d2_active {
            Ok(DeviceBatteryType::PercentVbat75_50)
        } else if d1_active {
            Ok(DeviceBatteryType::PercentVbat50_25)
        } else {
            Ok(DeviceBatteryType::PercentVbat25_0)
        }
    }

    pub fn is_charging(&self) -> bool {
        true
    }
}
