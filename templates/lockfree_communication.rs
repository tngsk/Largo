// lockfree_communication.rs
// 教育/カスタマイズ用テンプレート: ringbufを利用したスレッド間ロックフリー通信
// オーディオスレッドとメインスレッド間の安全なデータ受け渡しに使用します。

use ringbuf::traits::*;
use ringbuf::{SharedRb, storage::Heap};
use std::thread;
use std::time::Duration;

// 送信するコマンドの定義
#[derive(Debug)]
pub enum AudioCommand {
    SetParam { index: i32, value: f32 },
    Stop,
}

fn main() {
    // 1. ロックフリーバッファの初期化 (容量2048)
    let rb = SharedRb::<Heap<AudioCommand>>::new(2048);
    // producer: メインスレッド用 (送信側)
    // consumer: オーディオスレッド用 (受信側)
    let (mut producer, mut consumer) = rb.split();

    // 2. 受信側（オーディオスレッド相当）の開始
    let audio_thread = thread::spawn(move || {
        println!("[Audio Thread] Started.");
        loop {
            // ノンブロッキングでキューからポップ
            while let Some(cmd) = consumer.try_pop() {
                match cmd {
                    AudioCommand::SetParam { index, value } => {
                        println!(
                            "[Audio Thread] Processed command: SetParam index={}, value={}",
                            index, value
                        );
                    }
                    AudioCommand::Stop => {
                        println!("[Audio Thread] Received Stop command. Exiting...");
                        return;
                    }
                }
            }
            // 実際のオーディオスレッドはオーディオコールバックの周期で呼ばれるためsleepは不要ですが、
            // このサンプルではCPU負荷を下げるために少し待ちます。
            thread::sleep(Duration::from_millis(10));
        }
    });

    // 3. 送信側（メインスレッド相当）からのコマンド送信
    println!("[Main Thread] Sending commands...");

    producer
        .try_push(AudioCommand::SetParam {
            index: 0,
            value: 0.5,
        })
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    producer
        .try_push(AudioCommand::SetParam {
            index: 1,
            value: 1.2,
        })
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    producer.try_push(AudioCommand::Stop).unwrap();

    // 終了を待つ
    audio_thread.join().unwrap();
    println!("[Main Thread] Finished.");
}
