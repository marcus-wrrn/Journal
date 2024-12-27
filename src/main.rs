use std::env;
use std::process::Command;

const ENTRY_DIR: &str = "/home/marcuswrrn/Documents/entries";

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: open_vim <file>");
        std::process::exit(1);
    }

    let file = format!("{}/{}", ENTRY_DIR, &args[1]);

    let status = Command::new("vi").
        arg(file)
        .status()
        .expect("Failed to open Vi");

    if status.success() {
        println!("Exited Vi successfully.");
    } else {
        eprintln!("Did not close as expected");
        return;
    }


}
