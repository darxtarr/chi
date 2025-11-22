// spirit: The Chi compositor
// Phase 1: Receive bubbles over TCP, log them (no rendering yet)

use chi::{chi_lib_tlv_framing, chi_lib_message_types};
use std::io::{self, BufReader};
use std::net::{TcpListener, TcpStream};

fn handle_client(stream: TcpStream) -> io::Result<()> {
    let peer_addr = stream.peer_addr()?;
    println!("[spirit] Client connected: {}", peer_addr);

    let mut reader = BufReader::new(stream);
    let mut bubble_count = 0;

    loop {
        match chi_lib_tlv_framing::decode_frame(&mut reader) {
            Ok((msg_type, payload)) => {
                bubble_count += 1;
                println!("[spirit] Received bubble #{} from {}", bubble_count, peer_addr);

                match msg_type {
                    chi_lib_tlv_framing::MessageType::Text => {
                        match chi_lib_message_types::TextMessage::from_bytes(&payload) {
                            Ok(text_msg) => {
                                println!("  └─ Text: \"{}\" (color: #{:08X})",
                                    text_msg.content, text_msg.color_rgba);
                                // TODO: Create scene graph object, render to wgpu
                            }
                            Err(e) => eprintln!("  └─ ! Failed to decode text: {}", e),
                        }
                    }
                    chi_lib_tlv_framing::MessageType::Image => {
                        match chi_lib_message_types::ImageMessage::from_bytes(&payload) {
                            Ok(img_msg) => {
                                println!("  └─ Image: {}x{} ({} bytes)",
                                    img_msg.width, img_msg.height, img_msg.rgba_data.len());
                                // TODO: Upload to GPU texture, render to scene
                            }
                            Err(e) => eprintln!("  └─ ! Failed to decode image: {}", e),
                        }
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                println!("[spirit] Client {} disconnected ({} bubbles received)",
                    peer_addr, bubble_count);
                break;
            }
            Err(e) => {
                eprintln!("[spirit] Error reading from {}: {}", peer_addr, e);
                return Err(e);
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let listen_addr = "127.0.0.1:7777";
    let listener = TcpListener::bind(listen_addr)?;

    println!("[spirit] Chi compositor listening on {}", listen_addr);
    println!("[spirit] Waiting for bubbles from apps...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_client(stream) {
                    eprintln!("[spirit] Client handler error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[spirit] Connection error: {}", e);
            }
        }
    }

    Ok(())
}
