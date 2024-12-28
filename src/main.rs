use std::env;
use std::path::Path;
use std::process::Command;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs::OpenOptions;
use std::fs;
use std::io::Write;
use regex::Regex;


const ENTRY_DIR: &str = "/home/marcuswrrn/Documents/entries";

fn get_time() -> String {
    let dt = chrono::offset::Local::now();
    dt.to_rfc2822()
}

fn file_exists(filename: &str) -> bool {
    let path = Path::new(filename);
    path.exists()
}

fn initialize_file(filename: &str) {
    let current_date = get_time();
    let text = format!("{}\n\n=========================================================================================================\n", current_date);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
        .expect("Could not open file");

    file.write(text.as_bytes()).expect("Could not add text to file");
}

fn open_file(filename: &str) {
    if !file_exists(filename) {
        println!("Initializing file!");
        initialize_file(filename);
    }

    let status = Command::new("vim").
        arg(filename)
        .status()
        .expect("Failed to open Vim");

    if status.success() {
        println!("Exited Vim successfully.");
        //encrypt_file(filename);
    } else {
        eprintln!("Did not close as expected");
    }
}

fn delete_file(filename: &str) -> std::io::Result<()> {
    fs::remove_file(filename)?;
    Ok(())
}


// fn encrypt_file(filename: &str) {
//     let file = format!("{}/{}", ENTRY_DIR, filename);
//     println!("Encrypt File: {}", file);
//     //let cmd = format!("openssl enc -aes-256-cbc -salt -in {} -out {}", file, file);

//     let status = Command::new("openssl")
//         .arg("enc")
//         .arg("-aes-256-cbc")
//         .arg("-salt")
//         .arg("-in")
//         .arg(&file)
//         .arg("-out")
//         .arg(&file)
//         .status()
//         .expect("Failed to encrypt File");

//     if status.success() {
//         println!("File encrypted successfully");
//     } else {
//         eprintln!("Could not encrypt file");
//     }
// }

// fn decrypt_file(filename: &str) {
//     let file = format!("{}/{}", ENTRY_DIR, filename);
//     println!("Decrypt File: {}", file);
//     //let cmd = format!("openssl enc -d -aes-256-cbc -in {} -out {}", file, file);

//     let status = Command::new("openssl")
//         .arg("enc")
//         .arg("-d")
//         .arg("-aes-256-cbc")
//         .arg("-in")
//         .arg(&file)
//         .arg("-out")
//         .arg(&file)
//         .status()
//         .expect("Failed to Decrypt File");

//     if status.success() {
//         println!("File decrypted successfully");
//     } else {
//         eprintln!("Could not decrypt file");
//     }
// }

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
        let filename = format!("{}/Entry_{}.txt", ENTRY_DIR, largest + 1);
        open_file(&filename);
    } else {
        let filename = format!("{}/Entry_1.txt", ENTRY_DIR);
        open_file(&filename);
    }

}

fn edit_entry() {
    let mut files = get_files(ENTRY_DIR);

    if files.len() < 1 {
        println!("No files to edit");
        return;
    }

    files.push("Exit".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("=============Edit Files=============")
    .default(0)
    .items(&files)
    .interact()
    .expect("Failed to display edit menu");

    if selection == files.len() - 1 {
        return;
    }
    let filename = format!("{}/{}", ENTRY_DIR, &files[selection]);
    open_file(&filename);
}

fn delete_entry() {
    let mut files = get_files(ENTRY_DIR);

    if files.len() < 1 {
        println!("No files to edit");
        return;
    }

    files.push("Exit".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("=============Edit Files=============")
        .default(0)
        .items(&files)
        .interact()
        .expect("Failed to display delete menu");

    if selection == files.len() - 1 {
        return;
    }

    let filename = format!("{}/{}", ENTRY_DIR, &files[selection]);

    if let Err(e) = delete_file(&filename) {
        eprintln!("Failed to delete file: {}", e);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();  

    if args.len() >= 2 {
        open_file(&args[1]);
        return;
    }

    let options = vec!["Add Entry", "Edit Entry", "Delete Entry", "Exit"];  
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
        },
        2 => {
            delete_entry();
        },
        3 => {
            return;
        },
        _ => unreachable!(),
    }
}