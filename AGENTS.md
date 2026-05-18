# システム開発仕様書: Sound Design Engine

本仕様書は、Raspberry Pi 5（ARM64 Linux）環境における高信頼性・低レイテンシなエッジ音響処理、および現場での動的なコンポジションルール更新を可能にする音響エンジンの設計図である。他システム（Python等のセンサー処理層）とはOSC経由で連携する。

---

## 1. システム概要 & アーキテクチャ

システムは静的・超高速な「音響処理層（Rust/C++）」と、動的・高度に抽象化された「制御層（Lisp/Scheme）」を完全に分離したハイブリッド構成をとる。

* **Lisp/OSC制御スレッド（メイン）**: OSCメッセージの待ち受け、Lisp VMの実行、マッピングルールの動的評価、およびオーディオスレッドへのノンブロッキングなコマンド送信を担当。
* **全二重オーディオスレッド（リアルタイム）**: PipeWire/JACKを介した、サンプリングレート精度のオーディオ入出力、およびRNBO（C++）によるDSP処理を担当。**このスレッド内でのメモリ確保（Heap Allocation）、I/O、Lispコードの評価は完全に禁止する。**

---

## 2. ディレクトリ構造

```text
situ-core/
├── Cargo.toml
├── build.rs
├── rnbo_exported/
│   ├── RNBO.cpp
│   └── RNBO.h
├── scripts/
│   ├── main.scm                # コア・ディスパッチャ（不変）
│   └── plugins/
│       └── 01_sensor_synth.scm # ユーザー定義プラグイン（動的追加）
└── src/
    ├── main.rs                 # コアシステムループ、OSC、Lispバインディング
    └── rnbo_bridge.cpp         # C++ FFI ブリッジ

```

---

## 3. スレッド間通信プロトコル

### 3.1. メッセージ定義 (Rust)

Lispスレッドからオーディオスレッドへの単方向通信には、ロックフリー・リングバッファを介して以下の列挙型（Enum）を転送する。

```rust
pub enum AudioCommand {
    SetRnboParam {
        index: i32,
        value: f32,
    },
    Stop,
}

```

---

## 4. コンポーネント実装仕様 & サンプルコード

### 4.1. C++層: RNBOブリッジ (`src/rnbo_bridge.cpp`)

RNBOのC++クラスインスタンスをRustのFFI（Foreign Function Interface）経由で安全に制御するためのプレーンCラッパー。

```cpp
#include "RNBO.h"
#include <map>
#include <string>

extern "C" {
    // RNBO インスタンスの生成
    void* rnbo_create(double sample_rate, int block_size) {
        auto* obj = new RNBO::CoreObject();
        obj->prepareToProcess(sample_rate, block_size);
        return reinterpret_cast<void*>(obj);
    }

    // パラメータ名からインデックスへの変換（名前解決）
    int rnbo_get_param_index(void* ptr, const char* name) {
        auto* obj = reinterpret_cast<RNBO::CoreObject*>(ptr);
        RNBO::ParameterIndex idx = obj->getParameterIndexForID(name);
        if (idx == RNBO::INVALID_PARAMETER_INDEX) return -1;
        return static_cast<int>(idx);
    }

    // インデックス指定によるパラメータ更新
    void rnbo_set_parameter(void* ptr, int param_index, float value) {
        auto* obj = reinterpret_cast<RNBO::CoreObject*>(ptr);
        if (param_index >= 0) {
            obj->setParameterValue(param_index, value);
        }
    }

    // 全二重（同時入出力）プロセッシングループ
    void rnbo_process(void* ptr, const float* input, float* output, int num_samples) {
        auto* obj = reinterpret_cast<RNBO::CoreObject*>(ptr);
        
        // モノラル入出力を想定（ステレオの場合は適宜拡張）
        const float* inputs[] = { input };
        float* outputs[] = { output };
        
        obj->process(inputs, 1, outputs, 1, num_samples);
    }

    // インスタンスの破棄
    void rnbo_destroy(void* ptr) {
        delete reinterpret_cast<RNBO::CoreObject*>(ptr);
    }
}

```

### 4.2. Rust層: ビルド構成 (`build.rs`)

`cc` クレートを用い、C++ブリッジとRNBOエクスポートソースを静的ライブラリとして一括コンパイルする。

```rust
fn main() {
    println!("cargo:rerun-if-changed=src/rnbo_bridge.cpp");
    cc::Build::new()
        .cpp(true)
        .std("c++11")
        .file("src/rnbo_bridge.cpp")
        .file("rnbo_exported/RNBO.cpp")
        .include("rnbo_exported")
        .compile("rnbo_engine");
}

```

### 4.3. Rust層: コアエンジン (`src/main.rs`)

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{storage::Heap, traits::*, SharedRb};
use rosc::OscPacket;
use std::net::UdpSocket;
use std::sync::Arc;
use steel_interpreter::steel::SteelEngine;
use steel_interpreter::values::structs::UserDefinedStruct;

// FFI宣言
extern "C" {
    fn rnbo_create(sample_rate: f64, block_size: i32) -> *mut std::ffi::c_void;
    fn rnbo_get_param_index(ptr: *mut std::ffi::c_void, name: *const std::ffi::c_char) -> std::ffi::c_実現;
    fn rnbo_set_parameter(ptr: *mut std::ffi::c_void, param_index: std::ffi::c_int, value: f32);
    fn rnbo_process(ptr: *mut std::ffi::c_void, input: *const f32, output: *mut f32, num_samples: std::ffi::c_int);
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
        unsafe { rnbo_destroy(self.ptr); }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. ロックフリーバッファ初期化
    let rb = SharedRb::<Heap<AudioCommand>>::new(2048);
    let (mut producer, mut consumer) = rb.split();

    // 2. RNBO エンジンの初期化 (44.1kHz, BlockSize 256 として準備)
    let raw_rnbo_ptr = unsafe { rnbo_create(44100.0, 256) };
    let mut rnbo_container = SafeRnbo { ptr: raw_rnbo_ptr };
    let rnbo_ptr_clone = rnbo_container.ptr;

    // 3. CPAL 全二重オーディオスレッドの立ち上げ
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("No audio device found")?;
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(44100),
        buffer_size: cpal::BufferSize::Fixed(256),
    };

    let audio_stream = device.build_id_and_output_stream(
        &config,
        move |input: &[f32], output: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // コマンド回収（ノンブロッキング）
            while let Some(cmd) = consumer.pop() {
                match cmd {
                    AudioCommand::SetRnboParam { index, value } => {
                        unsafe { rnbo_set_parameter(rnbo_ptr_clone, index, value); }
                    }
                    AudioCommand::Stop => return,
                }
            }
            // 信号処理実行
            unsafe {
                rnbo_process(rnbo_ptr_clone, input.as_ptr(), output.as_mut_ptr(), output.len() as i32);
            }
            // プロactive対策：セーフティリミッター (クランプ処理)
            for sample in output.iter_mut() {
                *sample = sample.clamp(-0.95, 0.95);
            }
        },
        |err| epresentln!("Audio stream error: {}", err),
        None
    )?;
    audio_stream.play()?;

    // 4. Lisp (Steel) 駆動環境構築
    let mut lisp_engine = SteelEngine::new();
    
    // FFIラッパー関数のエクスポート: パラメータ名からIndexへのマップ解決とコマンド発行
    let raw_ptr_for_lisp = rnbo_container.ptr as usize;
    let mut prod_for_lisp = std::sync::Mutex::new(producer);

    lisp_engine.register_fn("sound:set-param", move |param_name: String, value: f64| {
        let c_name = std::ffi::CString::new(param_name).unwrap();
        let idx = unsafe { rnbo_get_param_index(raw_ptr_for_lisp as *mut std::ffi::c_void, c_name.as_ptr()) };
        if idx >= 0 {
            let mut p = prod_for_lisp.lock().unwrap();
            let _ = p.try_push(AudioCommand::SetRnboParam { index: idx, value: value as f32 });
        }
    });

    // スクリプトのロード
    lisp_engine.compile_and_run_raw_program(&std::fs::read_to_string("scripts/main.scm")?)?;

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
                        msg.args.iter().map(|a| match a {
                            rosc::OscType::Float(f) => f.to_string(),
                            rosc::OscType::Int(i) => i.to_string(),
                            _ => "0".to_string()
                        }).collect::<Vec<_>>().join(" ")
                    );
                    let _ = lisp_engine.compile_and_run_raw_program(&arg_string);
                }
            }
            Err(e) => eprintln!("OSC recv error: {}", e),
        }
    }
}

```

### 4.4. Lisp層: コア・ディスパッチャ (`scripts/main.scm`)

機能追加・修正時にも一切変更を加えない不変のルーティング層。

```scheme
;; =================================================================
;; 不変層: メイン・イベント・ディスパッチャ
;; =================================================================

(define *osc-hooks* '())

;; 各プラグインが初期化時に呼び出すレジストリAPI
(define (register-osc-hook! callback)
  (set! *osc-hooks* (cons callback *osc-hooks*)))

;; Rust側からメッセージ到達時に一律で叩かれる配信エントリポイント
(define (on-osc-event path args)
  (for-each (lambda (callback)
              (callback path args))
            *osc-hooks*))

;; プラグインスクリプトをここから評価（実際はディレクトリ走査推奨）
(load "scripts/plugins/01_sensor_synth.scm")

```

### 4.5. Lisp層: プラグインの具体例 (`scripts/plugins/01_sensor_synth.scm`)

新機能追加時は、この構造を持つファイルを独立して作成し、配置する。名前衝突を回避するために固有のプレフィックスを使用する規約とする。

```scheme
;; =================================================================
;; 拡張プラグイン: センサーマッピング & フィルター変調
;; =================================================================

;; プレフィックスを伴う関数定義
(define (plugin:map-range x in-min in-max out-min out-max)
  (+ out-min (* (/ (- x in-min) (- in-max in-min)) (- out-max out-min))))

;; 自律レジストリ登録（メインルーティングを汚染しない）
(register-osc-hook!
 (lambda (path args)
   (cond
     ;; センサーAのOSCアドレスをインターセプト
     ((string=? path "/sensor/distance")
      (let* ((raw-val (car args)) ; 0.0 から 1.0 の想定
             (target-freq (plugin:map-range raw-val 0.0 1.0 200.0 5000.0)))
        ;; Rust経由でRNBOの"cutoff"パラメータを操作
        (sound:set-param "cutoff" target-freq)))

     ;; 別のセンサーアドレスへのフック追加も、この条件節を増やすか別ファイルに分けるだけ
     ((string=? path "/sensor/intensity")
      (let ((gain (car args)))
        (sound:set-param "gain" gain)))

     (else #f))))

```

---

## 5. 運用およびプロactive安全対策

1. **メモリ解放の決定論的保証 (Rust-side):**
`SafeRnbo` 構造体への `Drop` トレイト実装により、プログラム終了時または Lisp VM 再起動による再初期化時に、C++側で確保したメモリ領域は自動で `delete` され、リークを完全に回避する。
2. **ハードウェア・プロテクション (Safety Limiter):**
オーディオスレッドの最最終段に実装した `.clamp(-0.95, 0.95)` により、RNBO内のフィードバック発振やLisp側のマッピング計算ミス（バグ）に起因する爆音パルスから、Raspberry Pi 5 に接続されたスピーカー等のアンプ機材・聴覚環境を物理的に保護する。
3. **スレッド優先度 (Linuxカーネル設定):**
本バイナリの運用時は、リアルタイム割り込みを許可するため、`/etc/security/limits.d/audio.conf` に `@audio - rtprio 95` の記述があることを必須要件とする。
