use std::process::exit;
use std::{env, io, str};

use std::time::Duration;

use std::net::TcpStream;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Write};

fn read_file(path: &str) -> BufReader<File> {
    // Attempts to open a file in read-only mode.
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => {
                    println!("File \"{path}\" not found !");
                    exit(-1)
                }
                _ => {
                    println!("Unknow error found !");
                    exit(-2)
                }
            };
        }
    };

    // Return what's inside the file
    BufReader::new(file)
}

const START_BLOCK: u8 = 0x0B;
const END_DATA: u8 = 0x0D;
const END_BLOCK: u8 = 0x1C;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut hl7_file = String::new();

    let ip = "127.0.0.1";
    let port = 7779;
    let mut hl7_server = format!("{}:{}", ip, port);

    match args.len() {
        // No need to check for 1 argument because we're checking all other cases below
        // One argument passed
        2 => {
            hl7_file.push_str(&args[1]);
        }
        // Two arguments passed
        3 => {
            hl7_file.push_str(&args[1]);
            hl7_server.clear();
            hl7_server.push_str(&args[2]);
        }
        // All the other cases
        _ => {
            println!("<HL7 file> and <IP:Port> are required !!!");
            exit(0);
        }
    }

    println!(">>> Connecting to {}", &hl7_server);
    if let Ok(mut stream) = TcpStream::connect(&hl7_server) {
        match stream.write(&[START_BLOCK]) {
            Ok(_) => {
                let hl7 = read_file(&hl7_file);
                for line in hl7.lines() {
                    let current_line = line.unwrap();
                    match stream.write(current_line.as_bytes()) {
                        // Sending the current line is Ok ?
                        // Then check if it's ok from server side, send END-DATA OK ?
                        Ok(_) => match stream.write(&[END_DATA]) {
                            // Just continue, do nothing here
                            Ok(_) => (),
                            Err(err) => {
                                println!("ERROR {} in checking 'END-DATA' sending to {}!", err, &hl7_server);
                                exit(-4)
                            }
                        },
                        Err(err) => {
                            println!("ERROR {} while sending file !", err);
                            exit(-3)
                        }
                    }
                    let _wait = Duration::from_millis(500);
                }

                match stream.write(&[END_BLOCK]) {
                    Ok(_) => match stream.write(&[END_DATA]) {
                        Ok(_) => (),
                        Err(err) => {
                            println!("ERROR {} in checking 'END-DATA' sending to {}!", err, &hl7_server);
                            exit(-4)
                        }
                    },
                    Err(err) => {
                        println!("ERROR {} in checking 'END-BLOCK' sending to {}!", err, &hl7_server);
                        exit(-5)
                    }
                }

                // *********************************************
                // *** Check acknowledgement from the server ***
                // *********************************************
                // Wrap the stream in a BufReader, so we can use the BufRead methods
                let mut acknowledge = BufReader::new(&mut stream);
                // Read current data in the TcpStream
                // It needs to be paired with the consume method to function properly.
                let received: Vec<u8> = acknowledge.fill_buf()?.to_vec();

                let length = received.len();
                // Ensure the bytes we worked with aren't returned again later
                // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
                acknowledge.consume(length);

                // Do some processing or validation to make sure the whole line is present?
                if length > 0 {
                    let message_ack =
                        String::from_utf8(received)
                            .map(|msg| format!("{}", msg))
                            .unwrap();

                    // A simple split to collect in an array (vector)
                    // and clean all what we don't want
                    let message: Vec<&str> = message_ack
                        .trim()
                        // If the pattern is a slice of chars, split on each occurrence of any of the characters:
                        .split(&['\u{0b}', '\u{1c}', '\r'][..])
                        .filter(|&text| !text.is_empty())
                        .collect();

                    // Just in case to be sure, get the separator after "MSH" in the first message
                    // 4th position so use index 3
                    let separator = message[0].chars().nth(3).unwrap();

                    // Then get the MSA segment, in the second line, so take message[1]
                    // Split the line with '|' and need to collect in a vector of string slide
                    // TODO: Check if segment MSA is present in "message"
                    let msa: Vec<&str> = message[1]
                        .split(separator)
                        .collect();

                    // The acknowledge reply is the 4th part in the line (index=3)
                    // "MSA|AA|13454277502|Message successfully received.|"
                    let msa_ack = match msa[1] {
                        "AA" => "Message successfully received.",
                        "AE" => "Problem processing the message, the sending application must correct the problem.",
                        "AR" => "Problem with field 9, field 11 or field 12 of the MSH segment of the incoming message.",
                        _ => "No acknowledge message received."
                    };

                    println!("<<< From {}: \"{}\"", &hl7_server, msa_ack);
                }
            }
            Err(_) => println!("Connexion refused or cannot connect !")
        }
    } else {
        println!("No connection could be established because the target computer expressly denied it or did not exist !");
    }
    Ok(())
}
