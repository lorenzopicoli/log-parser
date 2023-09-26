use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

struct InternalDataLog {
    internal_context: serde_any_json_todo,
    context: serde_anyjson_to,
    err: serde_any_json_todo_or_string,
}

struct StandardLog {
    level: u8,
    time: i64,
    pid: i32,
    hostname: String,
    name: String,
    data: InternalDataLog,
    msg: String,
}
// {"level":30,"time":1695749489844,"pid":55619,"hostname":"Lorenzos-MacBook-Pro.local","name":"[Worker-ID: 55619] - ","data":{"internalContext":{},"context":{}},"err":{},"msg":"HTTPClient - Executing request (GET) http://localhost:3013/groups/6664fe35-efa6-4a87-b69b-1fba30855453?"
// }

fn write_green() -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
    writeln!(&mut stdout, "green text!")
}

fn main() {
    println!("Hello, world!");
    write_green();
    println!("Hello world 3");
}
