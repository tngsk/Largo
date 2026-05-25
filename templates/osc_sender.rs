// osc_sender.rs
// 教育/カスタマイズ用テンプレート: UDP経由でのOSCメッセージ送信

use rosc::{OscMessage, OscPacket, OscType, encoder};
use std::net::UdpSocket;
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 任意の空きポートにバインド
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // 送信先のアドレスとポート
    let target_addr = "127.0.0.1:57120";
    println!("Sending OSC messages to {}...", target_addr);

    // 送信するメッセージの構築
    let address = "/sensor/val"; // アドレス
    let float_value = 0.5f32; // 引数

    let msg = OscPacket::Message(OscMessage {
        addr: address.to_string(),
        args: vec![OscType::Float(float_value)],
    });

    // メッセージのエンコード
    let buf = encoder::encode(&msg)?;

    // 送信ループ
    for i in 0..5 {
        println!("Sending {} {} (Count: {})", address, float_value, i);
        socket.send_to(&buf, target_addr)?;

        // 1秒待機
        thread::sleep(Duration::from_millis(1000));
    }

    println!("Finished sending.");
    Ok(())
}
