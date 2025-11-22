// bubble-dump: Network protocol inspector for Chi
// Reads TLV frames from stdin or file, pretty-prints in human-readable format

use chi::{chi_lib_tlv_framing, chi_lib_message_types};
use std::io::{self, stdin, BufReader};
use std::env;
use std::fs::File;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut reader: Box<dyn io::Read> = if args.len() > 1 {
        // Read from file
        let path = &args[1];
        Box::new(BufReader::new(File::open(path)?))
    } else {
        // Read from stdin
        Box::new(BufReader::new(stdin()))
    };

    let mut frame_count = 0;

    loop {
        match chi_lib_tlv_framing::decode_frame(&mut reader) {
            Ok((msg_type, payload)) => {
                frame_count += 1;
                println!("─── Frame #{} ───", frame_count);
                println!("Type: {:?}", msg_type);
                println!("Length: {} bytes", payload.len());

                // Decode specific message types
                match msg_type {
                    chi_lib_tlv_framing::MessageType::Text => {
                        match chi_lib_message_types::TextMessage::from_bytes(&payload) {
                            Ok(text_msg) => {
                                println!("Color: #{:08X}", text_msg.color_rgba);
                                println!("Content: \"{}\"", text_msg.content);
                            }
                            Err(e) => println!("! Failed to decode text: {}", e),
                        }
                    }
                    chi_lib_tlv_framing::MessageType::Image => {
                        match chi_lib_message_types::ImageMessage::from_bytes(&payload) {
                            Ok(img_msg) => {
                                println!("Dimensions: {}x{}", img_msg.width, img_msg.height);
                                println!("Data size: {} bytes", img_msg.rgba_data.len());
                            }
                            Err(e) => println!("! Failed to decode image: {}", e),
                        }
                    }
                }
                println!();
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // End of stream
                if frame_count == 0 {
                    eprintln!("No frames received.");
                } else {
                    eprintln!("─── End of stream ({} frames total) ───", frame_count);
                }
                break;
            }
            Err(e) => {
                eprintln!("! Error reading frame: {}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}
