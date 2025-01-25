use std::env;
use std::process::Command;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;

pub mod file_operations;
pub mod database;

use file_operations::{file_exists, initialize_file, get_files, extract_number};
use database::{initialize_database, PathConfig};

const ENTRY_DIR: &str = "/home/marcuswrrn/Documents/entries";

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
    let config = PathConfig::new(ENTRY_DIR);

    if args.len() >= 2 {
        if &args[1] == "init" {
            println!("Initializing Journal!");
            initialize_database(&config);
            return;
        }
        let filename = format!("{}/{}", ENTRY_DIR, &args[1]);
        open_file(&filename);
        return;
    }

    let options = vec!["Add Entry", "Edit Entry", "Delete Entry", "Exit"];  
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("=============Journal=============")
        .default(0)
        .items(&options)
        .interact()
        .expect("Failed to display menu");

    //let mut exit: bool = false;
    
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