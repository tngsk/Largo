use rosc::{encoder, OscMessage, OscPacket, OscType};
use std::net::UdpSocket;
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let target_addr = "127.0.0.1:57120";

    println!("Starting tutorial OSC sender...");
    println!("Sending OSC messages to {}...", target_addr);
    println!("Make sure the main engine is running (cargo run)");

    // Since `tutorial:map-param` calculates frequency as `100 + val * 100`,
    // we need to set `val = (freq - 100) / 100`.
    let notes = [
        (440.0 - 100.0) / 100.0, // A4
        (493.88 - 100.0) / 100.0, // B4
        (554.37 - 100.0) / 100.0, // C#5
        (587.33 - 100.0) / 100.0, // D5
        (659.25 - 100.0) / 100.0, // E5
        (587.33 - 100.0) / 100.0, // D5
        (554.37 - 100.0) / 100.0, // C#5
        (493.88 - 100.0) / 100.0, // B4
    ];

    for _ in 0..2 {
        for &val in &notes {
            let freq = 100.0 + val * 100.0;
            println!("Sending /demo/freq raw_val={}, (freq={})", val, freq);
            let msg = OscPacket::Message(OscMessage {
                addr: "/demo/freq".to_string(),
                args: vec![OscType::Float(val as f32)],
            });
            let buf = encoder::encode(&msg)?;
            socket.send_to(&buf, target_addr)?;

            println!("Sending /demo/amp 0.8");
            let amp_msg = OscPacket::Message(OscMessage {
                addr: "/demo/amp".to_string(),
                args: vec![OscType::Float(0.8)],
            });
            let amp_buf = encoder::encode(&amp_msg)?;
            socket.send_to(&amp_buf, target_addr)?;

            thread::sleep(Duration::from_millis(300));
        }
    }

    println!("Sending /demo/amp 0.0 to stop sound");
    let amp_msg = OscPacket::Message(OscMessage {
        addr: "/demo/amp".to_string(),
        args: vec![OscType::Float(0.0)],
    });
    let amp_buf = encoder::encode(&amp_msg)?;
    socket.send_to(&amp_buf, target_addr)?;

    println!("Tutorial demo finished.");
    Ok(())
}
