use esp_idf_svc::sntp::{EspSntp, SyncStatus};

pub fn set_ntp_time(timeout_s: u8, time_zone: &str) -> anyhow::Result<()> {
    if let Ok(sntp) = EspSntp::new_default() {
        log::info!("Waiting for system time to be set...");
        let mut total_wait = 0;
        while sntp.get_sync_status() != SyncStatus::Completed {
            std::thread::sleep(std::time::Duration::from_millis(500));
            total_wait += 500;
            if total_wait > timeout_s as u32 * 1000 {
                log::warn!("SNTP sync timeout, using local RTC");
                break;
            }
        }
        set_time_zone(time_zone)?;
    }
    Ok(())
}

pub fn set_time_zone(time_zone: &str) -> anyhow::Result<()> {
    unsafe {
        let env_name = std::ffi::CString::new("TZ")?;
        let env_val = std::ffi::CString::new(time_zone)?;
        esp_idf_sys::setenv(env_name.as_ptr(), env_val.as_ptr(), 1);
        esp_idf_sys::tzset();
        // log::info!("System time set to {}", time_zone);
    }
    Ok(())
}
