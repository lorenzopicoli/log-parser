use std::{
    error::Error,
    io::{self, BufRead, BufReader},
    process::{ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use termcolor::Color;

mod core;
mod logs;

#[derive(Debug)]
enum FormatType {
    Compact,
    Detailed,
}

#[derive(Debug)]
struct CliState {
    format_type: FormatType,
}

type Context = Arc<Mutex<CliState>>;

fn read_and_parse_logs(mut reader: BufReader<ChildStdout>, context: Context) {
    // Buffer that will hold lines as they come
    let mut buffer = String::new();
    // Keep trying to get a new line in a loop
    loop {
        // Try to read a new line
        match reader.read_line(&mut buffer) {
            Ok(_) => {
                let line = buffer.as_str();

                if line.len() == 0 {
                    buffer.clear();
                    continue;
                }
                // println!("{:#?}", context);
                if let Some(log) = logs::try_parse_known_log(line) {
                    let compact = match context.lock().unwrap().format_type {
                        FormatType::Compact => log.format_compact(),
                        FormatType::Detailed => log.format_detailed(),
                    };
                    compact.print();
                } else {
                    // Here we find unknown logs. Usually not formatted in JSON or quick
                    // console.logs.
                    // We should probably integrate this in parse_known_log and support other than
                    // compact logs
                    let compact = core::FormattedLog {
                        date: chrono::Local::now(),
                        msg: line.clone().to_string(),
                        extra: None,
                        color_overwrite: Some(Color::Blue),
                    };
                    compact.print();
                }
                buffer.clear()
            }
            Err(e) => {
                // Stop listening if something happened. Shouldn't be called
                println!("{}", e);
                break;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let child_shell = Command::new(&args[1])
        .args(&args[2..])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let context: Context = Arc::new(Mutex::new(CliState {
        format_type: FormatType::Detailed,
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
        if input.starts_with("logset:") {
            match input {
                "logset: compact" => {
                    context.lock().unwrap().format_type = FormatType::Compact;
                    println!("Setting mode to compact logs");
                }
                "logset: detailed" => {
                    context.lock().unwrap().format_type = FormatType::Detailed;
                    println!("Setting mode to detailed logs");
                }
                _ => println!("Unknown format"),
            }
            continue;
        }
    }

    child_thread.join().unwrap();

    Ok(())
}
