use std::env;
use std::process::Command;
use dialoguer::{theme::ColorfulTheme, Select};

pub mod file_operations;
pub mod database;

use file_operations::{file_exists, initialize_file, sort_entries_by_number, sort_entries_by_date, get_entry, Entry};
use database::{initialize_database, PathConfig};

const ENTRY_DIR: &str = "/home/marcus/Documents/entries";

fn open_file(filepath: &str) {
    if !file_exists(filepath) {
        println!("Initializing file!");
        initialize_file(filepath);
    }

    let status = Command::new("vim").
        arg(filepath)
        .status()
        .expect("Failed to open Vim");

    if status.success() {
        println!("Exited Vim successfully.");
        //encrypt_file(filename);
    } else {
        eprintln!("Did not close as expected");
    }
}

fn add_entry(path_config: &PathConfig) {
    let entry = Entry::create_default(path_config);
    entry.initialize(path_config);
    open_file(&entry.path);
}

fn edit_entry(path_config: &PathConfig) {
    let mut entries = Entry::get_entries(&path_config);
    sort_entries_by_number(&mut entries);

    if entries.len() < 1 {
        println!("No files to edit");
        return;
    }
    
    let mut filenames = entries.iter().map(|e| e.name.clone()).collect::<Vec<String>>();

    filenames.push("Exit".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("=============Edit Files=============")
    .default(0)
    .items(&filenames)
    .interact()
    .expect("Failed to display edit menu");

    if selection == filenames.len() - 1 {
        return;
    }
    let entry = &mut entries[selection];
    entry.update_entry(path_config);

    open_file(&entry.path);
}

fn delete_entry(path_config: &PathConfig) {
    let mut entries = Entry::get_entries(path_config);

    if entries.len() < 1 {
        println!("No files to edit");
        return;
    }

    let mut filenames = entries.iter().map(|e| e.name.clone()).collect::<Vec<String>>();
    filenames.push("Exit".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("=============Edit Files=============")
        .default(0)
        .items(&filenames)
        .interact()
        .expect("Failed to display delete menu");

    if selection == filenames.len() - 1 {
        return;
    }
    let entry = &mut entries[selection];
    entry.delete_entry(path_config);
}

fn last_accessed(path_config: &PathConfig) {
    let entries = Entry::get_entries(&path_config);

    // Filter out old entries
    let mut entries = entries.into_iter().filter_map(|e| {
        if e.access_date.is_some() {
            return Some(e);
        }
        None
    }).collect::<Vec<Entry>>();

    sort_entries_by_date(&mut entries, true);
    let index = entries.len() - 1;
    let entry = &mut entries[index];
    entry.update_entry(path_config);
    open_file(&entry.path);
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

        let entries = Entry::get_entries(&config);
        if let Some(mut entry) = get_entry(entries, &args[1]) {
            entry.update_entry(&config);
            open_file(&entry.name);
        } else {
            let entry = Entry::create_custom(&config, &args[1]);
            entry.initialize(&config);
            open_file(&entry.name);
        }
        
        return;
    }

    let mut selection = 0; 
    loop {
        let options = vec!["Last Accessed", "Add Entry", "Edit Entry", "Delete Entry", "Exit"];  
        selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("=============Journal=============")
            .default(selection)
            .items(&options)
            .interact()
            .expect("Failed to display menu");
    
        match  selection {
            0 => {
                last_accessed(&config);
            }
            1 => {
                add_entry(&config);
            },
            2 => {
                edit_entry(&config);
            },
            3 => {
                delete_entry(&config);
            },
            4 => {
                return;
            },
            _ => unreachable!(),
        }
    }
}