#[allow(dead_code)]
pub mod command;
pub mod driver;

use awedio::manager::Manager;
use std::time::Duration;

// const WAV_DATA: &[u8] =
//     include_bytes!("../../../assets/resource/test_resource/test_audio_44.1kHz.wav");

#[allow(dead_code)]
pub fn play_test_signal() -> anyhow::Result<Vec<u8>> {
    let mut buffer = Vec::with_capacity(4096);
    let amplitude: i16 = 20000; // 稍微降低一点防止削波噪声

    for i in 0..2048 {
        let val = if (i / 1024) % 2 == 0 {
            amplitude
        } else {
            -amplitude
        };
        let bytes = val.to_le_bytes();
        buffer.extend_from_slice(&bytes);
        buffer.extend_from_slice(&bytes);
    }
    Ok(buffer)
}

/// 播放正弦波音频
pub fn play_sine_wav(manager: &mut Manager, play_time_ms: u64) {
    log::info!("starting sine wav");
    let wave = awedio::sounds::SineWave::new(840.0);
    manager.play(Box::new(wave));
    std::thread::sleep(Duration::from_millis(play_time_ms));
    manager.clear();
    log::info!("stopping sine wav");
}
