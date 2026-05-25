// osc_receiver.rs
// 教育/カスタマイズ用テンプレート: UDP経由でのOSCメッセージ受信

use rosc::OscPacket;
use std::net::UdpSocket;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 待ち受けポートの指定
    let listen_addr = "127.0.0.1:57120";
    let socket = UdpSocket::bind(listen_addr)?;

    // 受信用のバッファ
    let mut buf = [0u8; rosc::decoder::MTU];

    println!("Listening for OSC messages on {}...", listen_addr);

    // 受信ループ
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                println!("Received packet with size {} from: {}", size, addr);
                // パケットのデコード
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size])?;
                handle_packet(packet);
            }
            Err(e) => {
                println!("Error receiving from socket: {}", e);
                break;
            }
        }
    }

    Ok(())
}

// パケットの処理関数
fn handle_packet(packet: OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            println!("OSC address: {}", msg.addr);
            for (i, arg) in msg.args.iter().enumerate() {
                println!("  Arg {}: {:?}", i, arg);
                // 必要に応じて引数の型チェックを行う
                // if let OscType::Float(f) = arg { ... }
            }
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC Bundle: {:?}", bundle);
            // Bundleの中身も再帰的に処理可能
        }
    }
}
