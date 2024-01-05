use std::process::exit;
use std::{env, io, str};

use std::net::TcpStream;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MshSegmentHeader {
    message_segment_header: String,
    field_separator: String,
    encoding_characters: String,
    sending_application: String,
    sending_facility: String,
    receiving_application: String,
    receiving_facility: String,
    date_time_message: String,
    security: String,
    message_type: String,
    message_control_id: String,
    processing_id: String,
    hl7_version: String,
    sequence_number: String,
    separation_pointer: String,
    accept_acknowledgement_type: String,
    application_acknowledgement_type: String,
    country_code: String,
    character_set: String,
    principle_language: String,
    alternative_character_set: String,
    message_profile_identifier: String,
}

// sob = chr(11)   # MLLP Start of Block Character
// eod = chr(28)   # MLLP End of Data Character
// eob = chr(13)   # MLLP End of Block Character

// ASCII:10 = LF - Line Feed
const LF: u8 = 0x0A;
// ASCII:13 = CR - Carriage Return
const CR: u8 = 0x0D;
const START_BLOCK_CHAR: u8 = 0x0B;
// ASCII:11 = VT - Vertical Tab
const END_DATA_CHAR_CR: u8 = 0x0D;
// ASCII:13 = CR - Carriage Return
const END_BLOCK_CHAR: u8 = 0x1C;     // ASCII:28 = FS - File Separator

fn is_segment(pat: &str, line: &str) -> bool {
    if line.trim().to_ascii_uppercase().starts_with(&pat.to_ascii_uppercase()) {
        true
    } else {
        false
    }
}

fn read_file(path: &str) -> String {
    // Attempts to open a file in read-only mode.
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => {
                    println!("File \"{}\" not found !", path);
                    exit(-1)
                }
                _ => {
                    println!("Unknow error found !\nFile: {}", path);
                    exit(-2)
                }
            };
        }
    };
    // Return what's inside the file
    let mut hl7_reader = BufReader::new(file);

    let mut vec_buffer = Vec::new();
    // read the whole file
    hl7_reader.read_to_end(&mut vec_buffer).expect("Cannot read to end in buffer");

    // Converts a vector of bytes to a String.
    let buffer =
        String::from_utf8(vec_buffer)
            // .map(|msg| format!("{}", msg))
            .unwrap();

    buffer
}

// TODO: Add Clap
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
        // write return how many bytes were written
        match stream.write(&[START_BLOCK_CHAR]) {
            // Byte written ?
            Ok(_) => {
                // Then send the file to the stream
                let buffer = read_file(&hl7_file);  // or hl7_file.as_str()

                // Collect all lines to get the first one to check if it's an ok HL7 file
                let hl7_lines: Vec<&str> = buffer
                    .trim()
                    // Split to force if only lines are separated by CR only (Mac format) or both
                    .split(&[LF as char, CR as char][..])
                    // If we have CRLF, then empty lines must be avoided
                    .filter(|&text| !text.is_empty())
                    .collect();

                // Now get the first line and check if it's a "MSH" segment
                let hl7_first_line = hl7_lines[0];
                if is_segment("msh", hl7_lines[0]) {
                    // Get the 4th char : "|" here
                    let field_separator = hl7_first_line.chars().nth(3).unwrap();
                    // Split the line with the separator extracted
                    let mut splitted: Vec<&str> = hl7_first_line.split(field_separator).collect();
                    // To add the separator, need to get it into a string
                    let field_separator = field_separator.to_string();
                    // and add it, at position 1, into the splitted vector as a &str (after segment MSH, pos 0)
                    splitted.insert(1, field_separator.as_str());
                    // Fix the size to 22, so the value we don't have from the splitted line will be equal to ""
                    splitted.resize(22, "");

                    let _msh = MshSegmentHeader {
                        message_segment_header: splitted[0].to_string(),
                        field_separator: splitted[1].to_string(),               // Manually inserted (cf above)
                        encoding_characters: splitted[2].to_string(),
                        sending_application: splitted[3].to_string(),
                        sending_facility: splitted[4].to_string(),
                        receiving_application: splitted[5].to_string(),
                        receiving_facility: splitted[6].to_string(),
                        date_time_message: splitted[7].to_string(),
                        security: splitted[8].to_string(),
                        message_type: splitted[9].to_string(),
                        message_control_id: splitted[10].to_string(),           // ID resend in the MSA
                        processing_id: splitted[11].to_string(),
                        hl7_version: splitted[12].to_string(),
                        sequence_number: splitted[13].to_string(),
                        separation_pointer: splitted[14].to_string(),
                        accept_acknowledgement_type: splitted[15].to_string(),
                        application_acknowledgement_type: splitted[16].to_string(),
                        country_code: splitted[17].to_string(),
                        character_set: splitted[18].to_string(),
                        principle_language: splitted[19].to_string(),
                        alternative_character_set: splitted[20].to_string(),
                        message_profile_identifier: splitted[21].to_string(),
                    };
                } else {
                    println!("ERROR with segment MSH in file {} !", &hl7_file);
                    exit(-3)
                }

                // If it's ok send the all buffer at once, no need to send each line because, sometimes
                // there is an issue with the sending in the stream, no CR is added
                stream.write(buffer.as_bytes())?;

                // Finished sending the file
                stream.write(&[END_BLOCK_CHAR])?;

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
                        // Force the constants to be as a char (from u8)
                        .split(&[START_BLOCK_CHAR as char, END_BLOCK_CHAR as char, END_DATA_CHAR_CR as char][..])
                        .filter(|&text| !text.is_empty())
                        .collect();

                    // Are we sure we have 2 lines ? (MSH and MSA)
                    if message.len() == 2 {
                        // And check if each line contains MSH AND MSA
                        if message[0].contains(&"MSH") && message[1].contains(&"MSA") {
                            // Just in case to be sure, get the separator after "MSH" in the first message
                            // 4th position so use index 3
                            let separator = message[0].chars().nth(3).unwrap();

                            // Then get the MSA segment, in the second line, so take message[1]
                            // Split the line with '|' and need to collect in a vector of string slide
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
                        } else {
                            println!("<<< From {}: \"No MSH and MSA segments received.\"", &hl7_server);
                            exit(-7);
                        }
                    } else {
                        println!("<<< From {}: \"No (enough?) messages received.\"", &hl7_server);
                        exit(-8);
                    }
                } else {
                    println!("<<< From {}: \"Nothing received.\"", &hl7_server);
                    exit(-9);
                }
            }
            Err(_) => {
                println!("Connexion refused or cannot connect !");
                exit(-10);
            }
        }
    } else {
        println!("No connection could be established because the target computer expressly denied it or did not exist !");
        exit(-11);
    }
    Ok(())
}
