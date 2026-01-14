use button_driver::{Button, ButtonConfig, Mode};
use esp_idf_svc::hal::gpio::{AnyInputPin, Input, PinDriver};
use std::thread::JoinHandle;
use std::time::Instant;

#[derive(Debug, PartialEq)]
pub enum KeyClickedType {
    NoClick,
    SingleClicked,
    DoubleClicked,
    TripleClicked,
}

/// 按下按键时发送的消息
#[derive(Debug, PartialEq)]
pub struct PressedKeyInfo {
    pub idx: usize,                 // 按键索引, 按照传入的容器顺序
    pub click_type: KeyClickedType, // 按下按键类型
}

/// 按键设备
#[derive(Debug)]
pub struct DeviceButton {
    pub read_thread_handle: Option<JoinHandle<()>>, // 按键线程, 需要被回收
}

impl DeviceButton {
    /// 通过vec快速初始化, 这样支持多个按键, 方便以后扩展
    pub fn new(
        key_pins: Vec<AnyInputPin>,
        tx: std::sync::mpsc::Sender<PressedKeyInfo>,
        exit: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> anyhow::Result<DeviceButton> {
        let mut keys = Vec::new();
        for key_pin in key_pins.into_iter() {
            let key = PinDriver::input(key_pin)?;
            let btn: Button<PinDriver<'_, _, _>, Instant> = Button::new(
                key,
                ButtonConfig {
                    mode: Mode::PullUp,
                    ..Default::default()
                },
            );
            keys.push(btn);
        }

        let handle = std::thread::spawn(move || {
            if let Err(e) = Self::key_run(keys, tx, exit) {
                log::error!("key run failed: {e:?}");
            }
        });

        Ok(DeviceButton {
            read_thread_handle: Some(handle),
        })
    }

    fn key_run(
        mut keys: Vec<Button<PinDriver<AnyInputPin, Input>, Instant>>,
        tx: std::sync::mpsc::Sender<PressedKeyInfo>,
        exit: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> anyhow::Result<()> {
        while !exit.load(std::sync::atomic::Ordering::Relaxed) {
            for (idx, key) in keys.iter_mut().enumerate() {
                key.tick();
                let mut click_type = KeyClickedType::NoClick;
                if key.is_clicked() {
                    click_type = KeyClickedType::SingleClicked;
                } else if key.is_double_clicked() {
                    click_type = KeyClickedType::DoubleClicked;
                } else if key.is_triple_clicked() {
                    click_type = KeyClickedType::TripleClicked;
                }
                if click_type != KeyClickedType::NoClick {
                    let key_msg = PressedKeyInfo { idx, click_type };
                    // log::info!("send key msg: {key_msg:?}");
                    if let Err(e) = tx.send(key_msg) {
                        log::error!("key msg send failed: {e:?}");
                    }
                }
                key.reset();
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::info!("key read thread exit");
        Ok(())
    }
}
impl Drop for DeviceButton {
    fn drop(&mut self) {
        if let Some(handle) = self.read_thread_handle.take() {
            log::info!("Waiting for key thread to join...");
            let _ = handle.join();
        }
    }
}
