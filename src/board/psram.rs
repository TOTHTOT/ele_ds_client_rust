pub fn check_psram() {
    unsafe {
        use esp_idf_svc::sys::{
            esp_get_free_internal_heap_size, heap_caps_get_free_size, heap_caps_get_total_size,
            MALLOC_CAP_SPIRAM,
        };

        let free_internal = esp_get_free_internal_heap_size();
        let free_psram = heap_caps_get_free_size(MALLOC_CAP_SPIRAM);
        let total_psram = heap_caps_get_total_size(MALLOC_CAP_SPIRAM);

        log::info!(
            "Memory Stats: DRAM Free: {} bytes | PSRAM: {} / {} bytes free",
            free_internal,
            free_psram,
            total_psram
        );
    }
}
