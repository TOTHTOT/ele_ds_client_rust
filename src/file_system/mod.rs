use anyhow::Context;
use esp_idf_svc::sys::*;
use std::ffi::CString;
use std::fs;
use std::io::{Read, Write};

pub fn nvs_flash_filesystem_init() -> anyhow::Result<()> {
    let mut first_init = false;
    log::info!("start init filesystem, nvs flash");
    unsafe {
        let ret = nvs_flash_init();
        if ret == ESP_ERR_NVS_NO_FREE_PAGES || ret == ESP_ERR_NVS_NEW_VERSION_FOUND {
            log::info!("fat partition need init");
            first_init = true;
            // 如果 nvs 需要擦除
            nvs_flash_erase();
            nvs_flash_init();
        } else {
            esp!(ret)?;
        }
    }

    // 启用磨损均衡功能
    let mut wl_handle = 0;
    let mount_config = esp_vfs_fat_mount_config_t {
        max_files: 5,
        format_if_mount_failed: true,
        allocation_unit_size: 4096,
        disk_status_check_enable: false,
        use_one_fat: false,
    };

    // 挂载 FAT 到 /fat, 分区 label 与 partitions.csv 中一致.
    let mount_point = String::from("/fat");
    let partition_label = String::from("storage");
    let res = unsafe {
        esp_vfs_fat_spiflash_mount(
            CString::new(mount_point)?.as_ptr(),
            CString::new(partition_label)?.as_ptr(),
            &mount_config,
            &mut wl_handle as *mut wl_handle_t,
        )
    };

    if res != esp_idf_svc::sys::ESP_OK {
        log::error!("esp_vfs_fat_spiflash_mount failed: {res}");
        return Err(anyhow::anyhow!(res));
    }
    log::info!("FAT mounted at /fat");
    if first_init {
        test_fs_rw()?;
    }
    Ok(())
}

fn test_fs_rw() -> anyhow::Result<()> {
    let path = "/fat/hello.txt";
    {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .context(format!("failed to create file: {path}"))?;
        f.write_all(b"hello from rust on esp32!\n")?;
    }
    let mut s = String::new();
    let mut f = std::fs::File::open(path)?;
    f.read_to_string(&mut s)?;
    log::info!("file content: {s}");
    Ok(())
}
