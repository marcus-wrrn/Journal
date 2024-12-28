use std::env;
use std::process::Command;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use regex::Regex;

const ENTRY_DIR: &str = "/home/marcuswrrn/Documents/entries";

fn open_file(filename: &str) {
    let file = format!("{}/{}", ENTRY_DIR, filename);

    let status = Command::new("vi").
        arg(file)
        .status()
        .expect("Failed to open Vi");

    if status.success() {
        println!("Exited Vi successfully.");
    } else {
        eprintln!("Did not close as expected");
    }
}

fn get_files(dir: &str) -> Vec<String> {
    let files = fs::read_dir(dir)
        .expect("Directory does not exist")
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().into_string().unwrap_or_default())
        .collect::<Vec<String>>();
    
    let mut sorted_entries = files.clone();
    sorted_entries.sort_by(|a, b| compare_filenames(a, b));
    sorted_entries
}

fn extract_number(x: &str) -> Option<u32> {
    let number_regex = Regex::new(r"\d+").unwrap();
    number_regex.find(x).and_then(|m| m.as_str().parse::<u32>().ok())
}

fn compare_filenames(a: &str, b: &str) -> std::cmp::Ordering {
    match (extract_number(a), extract_number(b)) {
        (Some(_), None) => std::cmp::Ordering::Greater, // Numbers come after non-numbers
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(num_a), Some(num_b)) => num_a.cmp(&num_b), // Compare numbers
        _ => a.cmp(b), // Fall back to lexicographical order
    }
}

fn add_entry() {
    let files = get_files(ENTRY_DIR);

    let numbers: Vec<u32> = files.iter().filter_map(|f|{
        extract_number(f)
    }).collect();

    if let Some(largest) = numbers.iter().max() {
        let filename = format!("Entry_{}.txt", largest + 1);
        open_file(&filename);
    } else {
        let filename = "Entry_1.txt";
        open_file(&filename);
    }

}

fn edit_entry() {
    let files = get_files(ENTRY_DIR);

    if files.len() < 1 {
        println!("No files to edit");
        return;
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("=============Edit Files=============")
    .default(0)
    .items(&files)
    .interact()
    .expect("Failed to display edit menu");

    open_file(&files[selection]);
}

fn main() {
    let args: Vec<String> = env::args().collect();  

    if args.len() >= 2 {
        open_file(&args[1]);
        return;
    }

    let options = vec!["Add Entry", "Edit Entry"];  
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("=============Journal=============")
        .default(0)
        .items(&options)
        .interact()
        .expect("Failed to display menu");

    
    match  selection {
        0 => {
            add_entry();

        },
        1 => {
            edit_entry();
        }
        _ => unreachable!(),
    }
}