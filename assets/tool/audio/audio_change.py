from pydub import AudioSegment
import os

def convert_to_esp32_wav(input_path, output_path):
    try:
        # 1. 加载音频文件
        audio = AudioSegment.from_file(input_path)

        # 2. 转换参数
        # frame_rate: 44100 (采样率)
        # sample_width: 2 (16位位深，2字节)
        # channels: 1 (单声道，如果你改为2就是双声道)
        audio = audio.set_frame_rate(44100).set_sample_width(2).set_channels(1)

        # 3. 导出为标准的 WAV
        audio.export(output_path, format="wav")
        print(f"成功转换: {input_path} -> {output_path}")
        print(f"参数: 44.1kHz, 16-bit, Mono")

    except Exception as e:
        print(f"转换失败: {e}")

if __name__ == "__main__":
    # 使用示例
    input_file = "./test_audio_raw.wav"  # 你的原始文件
    output_file = "test_audio.wav"
    convert_to_esp32_wav(input_file, output_file)