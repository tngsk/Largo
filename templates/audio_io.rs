// audio_io.rs
// 教育/カスタマイズ用テンプレート: cpalを利用した基本的なオーディオI/O
// 注: Linux環境では sudo apt-get install -y libasound2-dev が必要です。

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const SAMPLE_RATE: u32 = 44100;
const BLOCK_SIZE: u32 = 256;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. CPAL ホスト・デバイス初期化
    let host = cpal::default_host();

    let output_device = host
        .default_output_device()
        .ok_or("No output audio device found")?;

    let input_device = host
        .default_input_device()
        .ok_or("No input audio device found")?;

    println!("Output device: {}", output_device.name()?);
    println!("Input device: {}", input_device.name()?);

    // 2. ストリーム設定
    let config = cpal::StreamConfig {
        channels: 1, // モノラル
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Fixed(BLOCK_SIZE),
    };

    // 3. 入力ストリームの構築
    // ここでは入力データを無視していますが、実用時はリングバッファ等を用いて
    // 出力スレッドへ渡す実装が一般的です。
    let input_stream = input_device.build_input_stream(
        &config,
        move |_data: &[f32], _: &cpal::InputCallbackInfo| {
            // オーディオスレッドはロックフリーに保つ必要があります
            // println!等のブロッキング処理は避けるべきです
        },
        move |err| {
            eprintln!("Input stream error: {}", err);
        },
        None,
    )?;

    // 4. 出力ストリームの構築
    let mut phase: f32 = 0.0;
    let freq = 440.0;

    let output_stream = output_device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                // サイン波の生成
                *sample = (phase * 2.0 * std::f32::consts::PI).sin() * 0.1; // 音量を小さく
                phase = (phase + freq / SAMPLE_RATE as f32) % 1.0;

                // セーフティ機構: 過大入力を防止
                *sample = sample.clamp(-0.95, 0.95);
            }
        },
        move |err| {
            eprintln!("Output stream error: {}", err);
        },
        None,
    )?;

    // 5. 再生開始
    println!("Starting audio streams...");
    input_stream.play()?;
    output_stream.play()?;

    // 3秒間再生して終了
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("Finished playback.");

    Ok(())
}
