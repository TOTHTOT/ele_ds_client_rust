pub fn check_psram() {
    unsafe {
        let free_psram = esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_SPIRAM);
        let total_psram = esp_idf_sys::heap_caps_get_total_size(esp_idf_sys::MALLOC_CAP_SPIRAM);
        log::info!("PSRAM: {} / {} bytes free", free_psram, total_psram);
    }
}
