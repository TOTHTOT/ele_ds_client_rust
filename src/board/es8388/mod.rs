#[allow(dead_code)]
pub mod command;
pub mod driver;

pub fn generate_sine_wave(freq: f32, sample_rate: f32, duration_ms: usize) -> Vec<u8> {
    let num_samples = (sample_rate * (duration_ms as f32 / 1000.0)) as usize;
    let mut buf = Vec::with_capacity(num_samples * 2); // 16-bit = 2 bytes

    for i in 0..num_samples {
        // 计算正弦值 (-1.0 到 1.0)
        let sample = (2.0 * std::f32::consts::PI * freq * (i as f32) / sample_rate).sin();
        // 映射到 i16 范围
        let amplitude = (sample * 16384.0) as i16;
        // 转为小端字节序
        let bytes = amplitude.to_le_bytes();
        buf.push(bytes[0]);
        buf.push(bytes[1]);
    }
    buf
}
