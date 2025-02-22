use std::io::Write;
use std::{env, io};
use std::process::Command;
use dialoguer::{theme::ColorfulTheme, Select};

pub mod file_operations;
pub mod database;

use file_operations::{file_exists, initialize_file, sort_entries_by_number, sort_entries_by_date, get_entry, Entry};
use database::{EntryDB, PathConfig};

const ENTRY_DIR: &str = "/home/marcuswrrn/Documents/entries_test";

fn get_last_accessed(db: &EntryDB) -> Entry {
    let entries = db.get_entries();

    // Filter out old entries
    let mut entries = entries.into_iter().filter(|e| e.access_date.is_some()).collect::<Vec<Entry>>();

    sort_entries_by_date(&mut entries, true);
    
    entries.into_iter().last().expect("No valid entries found")
}

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

fn add_entry(db: &EntryDB) {
    let entry = db.create_default_entry();
    open_file(&entry.path);
}

fn edit_entry(db: &EntryDB) {
    let mut entries = db.get_entries();
    sort_entries_by_number(&mut entries);

    if entries.len() < 1 {
        println!("No files to edit");
        return;
    }
    
    let mut filenames = entries.iter().map(|e| e.name.clone()).collect::<Vec<String>>();

    filenames.push("Exit".to_string());

    let mut selection = 0;
    loop {
        selection = match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("=============Edit Files=============")
        .default(selection)
        .items(&filenames)
        .interact_opt() {
            Ok(Some(choice)) => choice,
            _ => return
        };

        if selection == filenames.len() - 1 {
            return;
        }
        let entry = &mut entries[selection];

        db.update_entry_access_date(entry);
        open_file(&entry.path);
    }
    
}


fn change_name(db: &EntryDB, entry: &mut Entry) {
    loop {
        let mut s = String::new();
        let mut action = String::new();
        print!("Enter new name for {}: ", &entry.name);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut s).expect("Did not enter a correct string");
        
        if s.trim().is_empty() || s.trim() == "^[" {
            return;
        }

        print!("Are you sure y/n (default y): ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut action).expect("Did not enter correct action");
        
        let action = action.trim();
        if action.trim() == "y" || action.trim().is_empty() {
            db.change_name(entry, &s.trim());
            return;
        }
    }
}

fn update_entry_name(db: &EntryDB) {
    let mut entries = db.get_entries();
    if entries.is_empty() {
        println!("No files to edit");
        return;
    }

    let mut filenames = entries.iter().map(|e| e.name.clone()).collect::<Vec<String>>();
    filenames.push("Exit".to_string());

    let mut selection = 0;
    loop {
        selection = match Select::with_theme(&ColorfulTheme::default())
            .with_prompt("=============Change Name=============")
            .default(selection)
            .items(&filenames)
            .interact_opt()
        {
            Ok(Some(choice)) => choice,
            _ => return,
        };
        
        if selection == filenames.len() - 1 {
            return;
        }

        // Ensure we get a fresh mutable reference to the selected entry
        if let Some(entry) = entries.get_mut(selection) {
            change_name(db, entry);
            filenames = entries.iter().map(|e| e.name.clone()).collect::<Vec<String>>();
            filenames.push(String::from("Exit"));
        }
    }
}

fn delete_entry(db: &EntryDB) {
    let mut selection = 0;
    loop {
        let mut entries = db.get_entries();

        if entries.len() < 1 {
            println!("No files to edit");
            return;
        }

        let mut filenames = entries.iter().map(|e| e.name.clone()).collect::<Vec<String>>();
        filenames.push("Exit".to_string());

        selection = match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("=============Edit Files=============")
        .default(selection)
        .items(&filenames)
        .interact_opt() {
            Ok(Some(choice)) => choice,
            _ => return
        };

        if selection == filenames.len() - 1 {
            return;
        }
        let entry = &mut entries[selection];
        db.delete_entry(entry);
        selection -= 1;
    }
    
}

fn last_accessed(db: &EntryDB) {
    let entry = &mut get_last_accessed(db);
    
    db.update_entry_access_date(entry);
    open_file(&entry.path);
}


fn argument_handling(args: &Vec<String>, db: &EntryDB) {
    match args[1].as_str() {
        "--rebuild_db" => {
            println!("Initializing Database!");
            db.rebuild_database();
        },
        "--init_tables" => {
            println!("Initializing tables");
            db.init_tables();
        }
        _ => {
            let entries = db.get_entries();
            if let Some(mut entry) = get_entry(entries, &args[1]) {
                db.update_entry_access_date(&mut entry);
                open_file(&entry.path);
            } else {
                let entry = db.create_custom_entry(&args[1]);
                open_file(&entry.path);
            }
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();  
    let db = EntryDB::new(PathConfig::new(ENTRY_DIR));

    if args.len() >= 2 {
        argument_handling(&args, &db);
        return;
    }

    let mut selection = 0; 
    let options = vec!["Last Accessed", "Add Entry", "Edit Entry", "Delete Entry", "Change Name", "Exit"];  
    loop {
        selection = match Select::with_theme(&ColorfulTheme::default())
            .with_prompt("=============Journal=============")
            .default(selection)
            .items(&options)
            .interact_opt() {
                Ok(Some(choice)) => choice,
                _ => return
            };

        match  selection {
            0 => {
                last_accessed(&db);
            },
            1 => {
                add_entry(&db);
            },
            2 => {
                edit_entry(&db);
            },
            3 => {
                delete_entry(&db);
            },
            4 => {
                update_entry_name(&db);
            },
            5 => {
                return;
            },
            _ => unreachable!(),
        }
    }
}