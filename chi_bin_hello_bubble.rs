// hello-bubble: Example Chi app
// Sends text bubbles to the Spirit over TCP

use chi::{chi_lib_tlv_framing, chi_lib_message_types};
use std::io;
use std::net::TcpStream;

fn main() -> io::Result<()> {
    let spirit_addr = "127.0.0.1:7777";

    eprintln!("[hello-bubble] Connecting to Spirit at {}...", spirit_addr);
    let mut stream = TcpStream::connect(spirit_addr)?;
    eprintln!("[hello-bubble] Connected!");

    // Create a text message (green color: 0x00FF00FF)
    let text_msg = chi_lib_message_types::TextMessage::new(
        "Hello from the app! This is semantic content.".to_string(),
        0x00FF00FF,
    );

    // Send to Spirit over TCP
    chi_lib_tlv_framing::write_frame(
        &mut stream,
        chi_lib_tlv_framing::MessageType::Text,
        &text_msg.to_bytes(),
    )?;

    eprintln!("[hello-bubble] Sent text bubble #1");

    // Send another message (blue color: 0x0000FFFF)
    let text_msg2 = chi_lib_message_types::TextMessage::new(
        "The Spirit will decide how to present this.".to_string(),
        0x0000FFFF,
    );

    chi_lib_tlv_framing::write_frame(
        &mut stream,
        chi_lib_tlv_framing::MessageType::Text,
        &text_msg2.to_bytes(),
    )?;

    eprintln!("[hello-bubble] Sent text bubble #2");
    eprintln!("[hello-bubble] Done. Spirit has our prayers.");

    Ok(())
}
