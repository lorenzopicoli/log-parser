use std::{
    io::{BufRead, BufReader},
    process::ChildStdout,
    sync::{Arc, Mutex},
};

use crate::logs;
use termcolor::Color;

const LOG_REPLAY_CAPACITY: usize = 10000;
// Useful to initialize empty array
const INITIALIZER: String = String::new();

#[derive(Debug)]
pub struct CliState {
    pub format_type: FormatType,
    pub last_logs: CircularBuffer,
}

impl CliState {
    pub fn insert_log(&mut self, log: String) {
        self.last_logs.push(log);
    }
    pub fn get_logs(&self) -> Vec<String> {
        self.last_logs.get_all()
    }
}

pub type Context = Arc<Mutex<CliState>>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum FormatType {
    Compact = 0,
    Detailed,
    Raw,
}


#[derive(Debug)]
pub struct CircularBuffer {
    buffer: [String; LOG_REPLAY_CAPACITY],
    index: usize,
    // Kinda bad to have this, there must be a better way, but it does the work
    looped: bool,
}


impl CircularBuffer {
    pub fn new() -> Self {
        CircularBuffer {
            buffer: [INITIALIZER; LOG_REPLAY_CAPACITY],
            index: 0,
            looped: false,
        }
    }

    fn push(&mut self, item: String) {
        self.buffer[self.index] = item;
        if self.index + 1 >= LOG_REPLAY_CAPACITY {
            self.looped = true;
        }
        self.index = (self.index + 1) % LOG_REPLAY_CAPACITY;
    }

    fn get_all(&self) -> Vec<String> {
        let mut vec = Vec::new();
        let lower = self.buffer[0..self.index].to_vec();

        // If we looped around at least once, then we need to start by the upper part of the array
        if self.looped {
            let upper = self.buffer[self.index..LOG_REPLAY_CAPACITY].to_vec();
            for el in upper.iter() {
                vec.push(el.to_string());
            }
        }
        for el in lower.iter() {
            vec.push(el.to_string());
        }

        return vec;
    }
}


/// Takes care of format and printing a line
fn handle_line(line: &str, format_type: &FormatType) {
    {
        if *format_type == FormatType::Raw {
            println!("{}", line);
            return;
        }
    }
    // println!("{:#?}", context);
    if let Some(log) = logs::try_parse_known_log(line) {
        let compact = match format_type {
            FormatType::Compact => log.format_compact(),
            FormatType::Detailed => log.format_detailed(),
            FormatType::Raw => {
                return;
            }
        };
        compact.print();
    } else {
        // Here we find unknown logs. Usually not formatted in JSON or quick
        // console.logs.
        // We should probably integrate this in parse_known_log and support other than
        // compact logs
        let compact = logs::FormattedLog {
            date: chrono::Local::now(),
            msg: line.clone().to_string(),
            extra: None,
            color_overwrite: Some(Color::Blue),
        };
        compact.print();
    }
}


pub fn read_and_parse_logs(mut reader: BufReader<ChildStdout>, context: Context) {
    // Buffer that will hold lines as they come
    let mut buffer = String::new();
    // Keep trying to get a new line in a loop
    loop {
        // Try to read a new line
        match reader.read_line(&mut buffer) {
            Ok(_) => {
                let line = buffer.as_str();
                if line.len() == 0 {
                    return;
                }
                let format_type = {
                    let mut lock = context.lock().unwrap();
                    lock.insert_log(line.to_string());
                    lock.format_type.clone()
                };
                handle_line(line, &format_type);
                buffer.clear();
            }
            Err(e) => {
                // Stop listening if something happened. Shouldn't be called
                println!("{}", e);
                break;
            }
        }
    }
}

pub fn replay(context: &Context) {
    let lock = context.lock().unwrap();
    for line in lock.get_logs() {
        let format_type = { lock.format_type.clone() };
        handle_line(line.as_str(), &format_type);
    }
}
