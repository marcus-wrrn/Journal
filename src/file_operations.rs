use std::{io::BufRead, path::Path};
use std::io;
use std::fs::OpenOptions;
use std::fs;
use std::io::Write;
use regex::Regex;
use chrono::{DateTime, FixedOffset};
use rusqlite::{Result, Row, Connection};
use crate::database::PathConfig;

fn get_time() -> String {
    let dt = chrono::offset::Local::now();
    dt.to_rfc2822()
}

pub fn file_exists(filename: &str) -> bool {
    let path = Path::new(filename);
    path.exists()
}

fn get_largest_number(conn: &Connection) -> u32 {
    let mut stmt = conn.prepare("SELECT MAX(number) FROM entries").expect("Entries not found");
    let largest_number: Option<u32> = stmt.query_row([], |row| row.get(0)).ok();
    if let Some(number) = largest_number {
        return number + 1;
    }
    1
}

pub fn initialize_file(filename: &str) {
    let current_date = get_time();
    let text = format!("{}\n\n=========================================================================================================\n", current_date);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
        .expect("Could not open file");

    file.write(text.as_bytes()).expect("Could not add text to file");
}

pub fn get_files(dir: &str) -> Vec<String> {
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

pub fn extract_number(x: &str) -> Option<u32> {
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

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub number: Option<u32>,
    pub entry_date: Option<DateTime<FixedOffset>>,
    pub access_date: Option<DateTime<FixedOffset>>
}

impl Entry {
    pub fn from_file(directory: &str, filename: &str) -> Self {
        let filepath = format!("{}/{}", directory, filename);
        let file = fs::File::open(&filepath).expect("Could not open file");
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            let entry_date = DateTime::parse_from_rfc2822(line.trim()).ok();
            return Self {
                name: filename.to_string(),
                path: filepath,
                number: extract_number(&filename),
                entry_date,
                access_date: None
            }
        }

        Self {
            name: filename.to_string(),
            path: filepath,
            number: extract_number(&filename),
            entry_date: None,
            access_date: None
        }
    }

    pub fn build_from_row(dir_path: &str, row: &Row) -> Result<Self> {
        let number: Option<u32> = row.get(0).ok();
        let name: String = row.get(1)?;
        let entry_date = match row.get::<_, String>(2) {
            Ok(val) => { DateTime::parse_from_rfc2822(&val.trim()).ok()},
            Err(_) => None, // Failed to fetch the value from the row
        };

        let access_date = match row.get::<_, String>(3) {
            Ok(val) => DateTime::parse_from_rfc2822(&val.trim()).ok(),
            Err(_) => None
        };

        let path = Path::new(dir_path).join(&name);
        let path = path.to_string_lossy().to_string();
        Ok(Self {
            name,
            number,
            entry_date,
            access_date,
            path
        })
    }

    pub fn create_default(path_config: &PathConfig) -> Self {
        let conn = Connection::open(&path_config.db).expect("Could not open DB");
        let number = get_largest_number(&conn);
        let entry_date = chrono::offset::Local::now();
        let access_date = chrono::offset::Local::now();
        let name = format!("Entry_{}.txt", number);
        let path = path_config.get_entry_path(&name);
        
        Self {
            name,
            number: Some(number),
            entry_date: Some(entry_date.into()),
            access_date: Some(access_date.into()),
            path
        }
    }

    pub fn create_custom(path_config: &PathConfig, name: &str) -> Self {
        let entry_date = chrono::offset::Local::now();
        let access_date = chrono::offset::Local::now();
        let path = path_config.get_entry_path(&name);

        Self {
            name: name.to_string(),
            number: None,
            entry_date: Some(entry_date.into()),
            access_date: Some(access_date.into()),
            path
        }
    }

    pub fn initialize(&self, path_config: &PathConfig) {
        if file_exists(&self.path) {
            return;
        }
        // Write text to file
        initialize_file(&self.path);

        // Add to db
        self.publish_entry(path_config);

    }

    pub fn delete_entry(&mut self, path_config: &PathConfig) {
        let conn = Connection::open(&path_config.db).expect("Could not open DB");
        // Delete from database
        conn.execute("DELETE FROM entries WHERE name = ?1", (self.name.clone(),)).expect("Could not delete entry from DB");
        // Delete file
        fs::remove_file(&self.path).expect("Could not delete file");
    }

    pub fn entry_string(&self) -> String {
        if let Some(val) = self.entry_date {
            return val.to_rfc2822();
        }
        "".to_string()
    }

    pub fn access_string(&self) -> String {
        if let Some(val) = self.access_date {
            return val.to_rfc2822();
        }
        "".to_string()
    }

    pub fn get_entries(path_config: &PathConfig) -> Vec<Entry> {
        let conn = Connection::open(&path_config.db).expect("Could not open database");
        let mut stmt = conn.prepare("SELECT * FROM entries").expect("Could not select entries in DB");
        let entries = stmt.query_map([], |row| {
            Entry::build_from_row(&path_config.entry_dir, &row)
        }).expect("Error reading entries");
    
        entries.into_iter()
                .filter_map(|val| val.ok())
                .collect::<Vec<Entry>>()
    }

    pub fn update_entry(&mut self, path_config: &PathConfig) {
        let conn = Connection::open(&path_config.db).expect("Could not open DB");
        self.access_date = Some(chrono::offset::Local::now().into());
        if let Some(date) = self.access_date {
            conn.execute(
                "UPDATE entries SET access_date = ?1 WHERE name = ?2", 
                (date.to_rfc2822(), self.name.clone()))
                .expect("Could not update access_date in DB");
        }
    }

    fn publish_entry(&self, path_config: &PathConfig) {
        let conn = Connection::open(&path_config.db).expect("Could not open DB when adding entry");

        let entry_string = self.entry_string();
        let access_string = self.access_string();
        if let Some(number) = self.number {
            conn.execute(
                "INSERT INTO entries (number, name, entry_date, access_date) VALUES (?1, ?2, ?3, ?4)", 
                (number, self.name.clone(), entry_string, access_string)).expect("Could not add entry to DB");
        } else {
            conn.execute(
                "INSERT INTO entries (name, entry_date, access_date) VALUES (?1, ?2, ?3)", 
                (self.name.clone(), entry_string, access_string)).expect("Could not add entry to DB");
        }
    }
}

// Function to sort entries by `entry_date`
pub fn sort_entries_by_date(entries: &mut Vec<Entry>, use_access: bool) {
    entries.sort_by(|a, b| {
        let a_entry = if use_access { a.access_date } else { a.entry_date };
        let b_entry = if use_access { b.access_date } else { b.entry_date };

        match (a_entry, b_entry) {
            (Some(a_date), Some(b_date)) => a_date.cmp(&b_date), // Compare dates if both are present
            (Some(_), None) => std::cmp::Ordering::Less,         // Entries with a date come first
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,           // Both dates are None
        }
    });
}

pub fn get_entry(entries: Vec<Entry>, name: &str) -> Option<Entry> {
    for entry in entries {
        if name == entry.name {
            return Some(entry);
        }
    }
    None
}

pub fn sort_entries_by_number(entries: &mut Vec<Entry>) {
    entries.sort_by(|a, b| {
        match (a.number, b.number) {
            (Some(a_num), Some(b_num)) => a_num.cmp(&b_num), // Compare dates if both are present
            (Some(_), None) => std::cmp::Ordering::Less,         // Entries with a date come first
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,           // Both dates are None
        }
    });
}
