use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::*;
use ringbuf::{storage::Heap, SharedRb};
use rosc::OscPacket;
use std::net::UdpSocket;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use steel_core::steel_vm::engine::Engine;
use steel_core::steel_vm::register_fn::RegisterFn;

const BLOCK_SIZE: usize = 256;

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
    let (producer, mut consumer) = rb.split();

    // 1.5. オーディオ入力用リングバッファの初期化
    let audio_rb = SharedRb::<Heap<f32>>::new(BLOCK_SIZE * 4);
    let (mut audio_producer, mut audio_consumer) = audio_rb.split();

    // 2. RNBO エンジンの初期化
    let raw_rnbo_ptr = unsafe { rnbo_create(44100.0, BLOCK_SIZE as i32) };
    let rnbo_container = SafeRnbo { ptr: raw_rnbo_ptr };
    let rnbo_ptr_clone = rnbo_container.ptr as usize;

    // 3. CPAL ホスト・デバイス初期化
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No audio device found")?;

    let input_device = host
        .default_input_device()
        .ok_or("No input audio device found")?;

    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(44100),
        buffer_size: cpal::BufferSize::Fixed(BLOCK_SIZE as u32),
    };

    let audio_error_flag = Arc::new(AtomicBool::new(false));
    let audio_error_flag_clone = audio_error_flag.clone();
    let audio_error_flag_input = audio_error_flag.clone();

    let audio_stop_flag = Arc::new(AtomicBool::new(false));
    let audio_stop_flag_clone = audio_stop_flag.clone();

    let _audio_input_stream = input_device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for &sample in data {
                let _ = audio_producer.try_push(sample);
            }
        },
        move |_err| {
            audio_error_flag_input.store(true, Ordering::SeqCst);
        },
        None,
    )?;
    _audio_input_stream.play()?;

    let audio_stream = device.build_output_stream(
        &config,
        move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut stop_requested = false;
            while let Some(cmd) = consumer.try_pop() {
                match cmd {
                    AudioCommand::SetRnboParam { index, value } => unsafe {
                        rnbo_set_parameter(rnbo_ptr_clone as *mut std::ffi::c_void, index, value);
                    },
                    AudioCommand::Stop => {
                        stop_requested = true;
                    }
                }
            }

            if stop_requested {
                audio_stop_flag_clone.store(true, Ordering::SeqCst);
                for sample in output.iter_mut() {
                    *sample = 0.0;
                }
                return;
            }

            // 入力バッファをスタック上に用意
            let mut input_buffer = [0.0f32; BLOCK_SIZE];
            let process_len = std::cmp::min(output.len(), BLOCK_SIZE);

            // リングバッファからサンプルを取り出し、足りない場合は0で埋める
            for i in 0..process_len {
                input_buffer[i] = audio_consumer.try_pop().unwrap_or(0.0f32);
            }

            unsafe {
                rnbo_process(
                    rnbo_ptr_clone as *mut std::ffi::c_void,
                    input_buffer.as_ptr(),
                    output.as_mut_ptr(),
                    process_len as i32,
                );
            }
            for sample in output.iter_mut() {
                *sample = sample.clamp(-0.95, 0.95);
            }
        },
        move |_err| {
            audio_error_flag_clone.store(true, Ordering::SeqCst);
        },
        None,
    )?;
    audio_stream.play()?;

    // 4. Lisp (Steel) 駆動環境構築
    let mut lisp_engine = Engine::new();
    let raw_ptr_for_lisp = rnbo_container.ptr as usize;
    let prod_for_lisp = std::sync::Mutex::new(producer);

    lisp_engine.register_fn("sound:set-param", move |param_name: String, value: f64| {
        let c_name = std::ffi::CString::new(param_name.clone()).unwrap();
        let idx = unsafe {
            rnbo_get_param_index(raw_ptr_for_lisp as *mut std::ffi::c_void, c_name.as_ptr())
        };
        if idx >= 0 {
            println!(
                "Lisp call -> sound:set-param: {} (idx: {}) = {}",
                param_name, idx, value
            );
            let mut p = prod_for_lisp.lock().unwrap();
            let _ = p.try_push(AudioCommand::SetRnboParam {
                index: idx,
                value: value as f32,
            });
        } else {
            eprintln!("Unknown parameter: {}", param_name);
        }
    });

    // スクリプトのロード（パス自動判別）
    let main_scm_path = if std::path::Path::new("core/scripts/main.scm").exists() {
        "core/scripts/main.scm"
    } else {
        "scripts/main.scm"
    };
    let script = std::fs::read_to_string(main_scm_path)?;
    lisp_engine.compile_and_run_raw_program(script)?;

    // プラグインディレクトリの走査と読み込み
    let plugins_dir = if std::path::Path::new("core/scripts/plugins").exists() {
        "core/scripts/plugins"
    } else {
        "scripts/plugins"
    };
    if let Ok(entries) = std::fs::read_dir(plugins_dir) {
        let paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "scm"))
            .collect();

        let mut files_map = std::collections::HashMap::new();
        let mut dependencies = std::collections::HashMap::new();
        let mut in_degree = std::collections::HashMap::new();

        for path in &paths {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                let filename_string = filename.to_string();
                files_map.insert(filename_string.clone(), path.clone());
                in_degree.entry(filename_string.clone()).or_insert(0);

                if let Ok(content) = std::fs::read_to_string(path) {
                    let reqs: Vec<String> = content
                        .lines()
                        .filter(|l| l.starts_with(";; @require:"))
                        .map(|l| l.trim_start_matches(";; @require:").trim().to_string())
                        .collect();

                    for req in reqs {
                        dependencies
                            .entry(req.clone())
                            .or_insert_with(Vec::new)
                            .push(filename_string.clone());
                        *in_degree.entry(filename_string.clone()).or_insert(0) += 1;
                        in_degree.entry(req.clone()).or_insert(0);
                    }
                }
            }
        }

        let mut zero_in_degree: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut sorted_files = Vec::new();

        while !zero_in_degree.is_empty() {
            zero_in_degree.sort(); // 同一優先度の場合は名前順
            let current = zero_in_degree.remove(0);

            if let Some(path) = files_map.get(&current) {
                sorted_files.push(path.clone());
            }

            if let Some(neighbors) = dependencies.get(&current) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            zero_in_degree.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        // 循環参照等でロードできなかったファイルがあれば、末尾に名前順で追加してフォールバック
        if sorted_files.len() < files_map.len() {
            eprintln!("Warning: circular dependency detected in plugins. Falling back to alphabetical order for remaining files.");
            let mut remaining: Vec<_> = files_map
                .into_iter()
                .filter(|(_, path)| !sorted_files.contains(path))
                .collect();
            remaining.sort_by_key(|(name, _)| name.clone());
            for (_, path) in remaining {
                sorted_files.push(path);
            }
        }

        for path in sorted_files {
            if let Some(path_str) = path.to_str() {
                let path_str_normalized = path_str.replace("\\", "/");
                println!("Loading plugin: {}", path_str_normalized);
                let load_expr = format!("(load \"{}\")", path_str_normalized);
                if let Err(e) = lisp_engine.compile_and_run_raw_program(load_expr) {
                    eprintln!("Error loading plugin {}: {:?}", path_str_normalized, e);
                }
            }
        }
    }

    // 5. OSC受信制御ループ
    let socket = UdpSocket::bind("127.0.0.1:57120")?;
    socket.set_read_timeout(Some(std::time::Duration::from_millis(100)))?;
    let mut buf = [0u8; 1024];

    println!("Largo Core started (OSC: 127.0.0.1:57120)");
    loop {
        if audio_error_flag.load(Ordering::SeqCst) {
            eprintln!("Audio stream error occurred.");
            audio_error_flag.store(false, Ordering::SeqCst);
        }

        if audio_stop_flag.load(Ordering::SeqCst) {
            println!("Audio stream stop requested. Exiting core loop.");
            break Ok(());
        }

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
                    if let Err(e) = lisp_engine.compile_and_run_raw_program(arg_string) {
                        eprintln!("Lisp evaluation error on OSC event: {:?}", e);
                    }
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock
                    && e.kind() != std::io::ErrorKind::TimedOut
                {
                    eprintln!("OSC recv error: {}", e);
                }
            }
        }
    }
}
