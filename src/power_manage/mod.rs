use esp_idf_svc::sys::*;

pub fn enter_sleep_mode() -> anyhow::Result<()> {
    let wakeup_time_us = 10 * 1000 * 1000;
    unsafe {
        log::info!("sleeping for {} us", wakeup_time_us);
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

    log::info!("wakeup sleep time: {} ms", actual_sleep_ms);
    return Ok(());
}
