use crate::core::{read_and_parse_logs, replay, CircularBuffer, CliState, Context, FormatType};
use std::{
    error::Error,
    io::{self, BufReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

mod core;
mod logs;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let child_shell = Command::new(&args[1])
        .args(&args[2..])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let context: Context = Arc::new(Mutex::new(CliState {
        format_type: FormatType::Detailed,
        last_logs: CircularBuffer::new(),
    }));
    let child_out = BufReader::new(child_shell.stdout.unwrap());

    // So the main thread isn't hanging waiting for the server
    let c = context.clone();
    let child_thread = thread::spawn(move || read_and_parse_logs(child_out, c));

    loop {
        // Read user input
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        // Trim whitespace and newlines from the input
        let input = input.trim();
        if input.to_lowercase() == "exit" {
            break;
        }
        if input.to_lowercase() == "a" {
            println!("logs are {:#?}", context.lock().unwrap().get_logs());
        }
        if input.starts_with("c:") {
            match input.replace("c:", "").trim() {
                "compact" => {
                    context.lock().unwrap().format_type = FormatType::Compact;
                    println!("Setting mode to compact logs");
                }
                "detailed" => {
                    context.lock().unwrap().format_type = FormatType::Detailed;
                    println!("Setting mode to detailed logs");
                }
                "raw" => {
                    context.lock().unwrap().format_type = FormatType::Raw;
                    println!("Setting mode to raw logs");
                }
                "replay" => {
                    // Clear terminal first
                    print!("{esc}c", esc = 27 as char);
                    println!("--------- Replaying ----------");
                    replay(&context);
                }
                _ => println!("Unknown format"),
            }
            continue;
        }
    }

    child_thread.join().unwrap();

    Ok(())
}
