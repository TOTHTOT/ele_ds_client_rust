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
