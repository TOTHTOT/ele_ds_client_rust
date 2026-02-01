use awedio::manager::Manager;
use awedio::sounds;
use awedio::Sound;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub enum AudioCmd {
    Beep(u32, u64), // 发出蜂鸣器声音, (响铃次数, 时间_ms)
    Music(String),  // 播放音乐, 路径
}

/// 扬声器线程, 处理各种音频事件
pub fn speaker_task(
    mut manager: Manager,
    rx: std::sync::mpsc::Receiver<AudioCmd>,
    exit: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let beep_run_end = Arc::new(AtomicBool::new(true));
    while !exit.load(std::sync::atomic::Ordering::Relaxed) {
        let Ok(rx_cmd) = rx.recv() else {
            continue;
        };
        log::info!("rx_cmd: {rx_cmd:?}");
        match rx_cmd {
            AudioCmd::Beep(times, duration) => {
                log::info!("beep: {times:?}, duration: {duration}");
                if beep_run_end.load(std::sync::atomic::Ordering::Relaxed) {
                    play_button_beep(&mut manager, times, duration, beep_run_end.clone());
                } else {
                    log::info!("beep still running");
                }
            }
            AudioCmd::Music(path) => {
                log::info!("music: {path}");
                let (sound, _notifier) = sounds::open_file_with_buffer_capacity(path, 16 * 1024)?
                    .with_completion_notifier();

                manager.play(Box::new(sound));
                // manager.play(Box::new(sound));
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    log::info!("speaker_task exit");
    Ok(())
}

/// 播放正弦波音频
pub fn play_sine_wav(manager: &mut Manager, play_time_ms: u64) {
    log::info!("starting sine wav");
    let wave = sounds::SineWave::new(840.0);
    manager.play(Box::new(wave));
    std::thread::sleep(Duration::from_millis(play_time_ms));
    manager.clear();
    log::info!("stopping sine wav");
}

pub fn play_button_beep(manager: &mut Manager, times: u32, duration_ms: u64, end: Arc<AtomicBool>) {
    end.store(false, std::sync::atomic::Ordering::Relaxed);

    let beep = sounds::SineWave::new(2700.0);
    manager.play(Box::new(beep));

    let mut m = manager.clone();
    std::thread::spawn(move || {
        for _ in 0..times {
            std::thread::sleep(Duration::from_millis(duration_ms));
        }
        m.clear();
        end.store(true, std::sync::atomic::Ordering::Relaxed);
    });
}
