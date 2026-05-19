use steel_core::steel_vm::register_fn::RegisterFn;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::*;
use ringbuf::{storage::Heap, SharedRb};
use rosc::OscPacket;
use std::net::UdpSocket;
use steel_core::steel_vm::engine::Engine;

// FFI宣言
extern "C" {
    fn rnbo_create(sample_rate: f64, block_size: i32) -> *mut std::ffi::c_void;
    fn rnbo_get_param_index(
        ptr: *mut std::ffi::c_void,
        name: *const std::ffi::c_char,
    ) -> std::ffi::c_int;
    fn rnbo_set_parameter(ptr: *mut std::ffi::c_void, param_index: std::ffi::c_int, value: f32);
    fn rnbo_process(
        ptr: *mut std::ffi::c_void,
        input: *const f32,
        output: *mut f32,
        num_samples: std::ffi::c_int,
    );
    fn rnbo_destroy(ptr: *mut std::ffi::c_void);
}

pub enum AudioCommand {
    SetRnboParam { index: i32, value: f32 },
    Stop,
}

struct SafeRnbo {
    ptr: *mut std::ffi::c_void,
}
unsafe impl Send for SafeRnbo {}

impl Drop for SafeRnbo {
    fn drop(&mut self) {
        unsafe {
            rnbo_destroy(self.ptr);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. ロックフリーバッファ初期化
    let rb = SharedRb::<Heap<AudioCommand>>::new(2048);
    let (mut producer, mut consumer) = rb.split();

    // 2. RNBO エンジンの初期化 (44.1kHz, BlockSize 256 として準備)
    let raw_rnbo_ptr = unsafe { rnbo_create(44100.0, 256) };
    let mut rnbo_container = SafeRnbo { ptr: raw_rnbo_ptr };
    let rnbo_ptr_clone = rnbo_container.ptr as usize;

    // 3. CPAL 全二重オーディオスレッドの立ち上げ
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No audio device found")?;
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(44100),
        buffer_size: cpal::BufferSize::Fixed(256),
    };

    let audio_stream = device.build_output_stream(
        &config,
        move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // コマンド回収（ノンブロッキング）
            while let Some(cmd) = consumer.try_pop() {
                match cmd {
                    AudioCommand::SetRnboParam { index, value } => unsafe {
                        rnbo_set_parameter(rnbo_ptr_clone as *mut std::ffi::c_void, index, value);
                    },
                    AudioCommand::Stop => return,
                }
            }
            // 信号処理実行。本来は全二重だが、ここでは出力ストリームとしてモック
            let input_buf = vec![0.0; output.len()];
            unsafe {
                rnbo_process(
                    rnbo_ptr_clone as *mut std::ffi::c_void,
                    input_buf.as_ptr(),
                    output.as_mut_ptr(),
                    output.len() as i32,
                );
            }
            // プロactive対策：セーフティリミッター (クランプ処理)
            for sample in output.iter_mut() {
                *sample = sample.clamp(-0.95, 0.95);
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;
    audio_stream.play()?;

    // 4. Lisp (Steel) 駆動環境構築
    let mut lisp_engine = Engine::new();

    // FFIラッパー関数のエクスポート: パラメータ名からIndexへのマップ解決とコマンド発行
    let raw_ptr_for_lisp = rnbo_container.ptr as usize;
    let prod_for_lisp = std::sync::Mutex::new(producer);

    lisp_engine.register_fn("sound:set-param", move |param_name: String, value: f64| {
        let c_name = std::ffi::CString::new(param_name).unwrap();
        let idx = unsafe {
            rnbo_get_param_index(raw_ptr_for_lisp as *mut std::ffi::c_void, c_name.as_ptr())
        };
        if idx >= 0 {
            let mut p = prod_for_lisp.lock().unwrap();
            let _ = p.try_push(AudioCommand::SetRnboParam {
                index: idx,
                value: value as f32,
            });
        }
    });

    // スクリプトのロード
    let script = std::fs::read_to_string("scripts/main.scm")?;
    lisp_engine.compile_and_run_raw_program(script)?;

    // 5. OSC受信制御ループ（メインスレッド）
    let socket = UdpSocket::bind("127.0.0.1:57120")?;
    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, _)) => {
                if let Ok((_, OscPacket::Message(msg))) = rosc::decoder::decode_udp(&buf[..amt]) {
                    // Lispへのイベント配信
                    let arg_string = format!(
                        "(on-osc-event \"{}\" '({}))",
                        msg.addr,
                        msg.args
                            .iter()
                            .map(|a| match a {
                                rosc::OscType::Float(f) => f.to_string(),
                                rosc::OscType::Int(i) => i.to_string(),
                                _ => "0".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                    let _ = lisp_engine.compile_and_run_raw_program(arg_string);
                }
            }
            Err(e) => eprintln!("OSC recv error: {}", e),
        }
    }
}