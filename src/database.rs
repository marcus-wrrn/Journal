use rusqlite::{Connection, Result};
use std::path::Path;
use std::fs;
use crate::file_operations::{sort_entries_by_number, Entry};

pub struct PathConfig {
    pub db: String,
    pub entry_dir: String,
    pub main_dir: String,
}

impl PathConfig {
    pub fn new(base_dir: &str) -> Self {
        let db_path = Path::new(base_dir).join("db.sqlite");
        let entry_dir = Path::new(base_dir).join("entries");
        let main_dir = Path::new(base_dir);

        Self {
            db: db_path.to_string_lossy().to_string(),
            entry_dir: entry_dir.to_string_lossy().to_string(),
            main_dir: main_dir.to_string_lossy().to_string(),
        }
    }

    pub fn get_file(&self) -> Vec<String> {
        fs::read_dir(&self.entry_dir)
        .expect("Directory Does not exist")
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().into_string().unwrap_or_default())
        .collect::<Vec<String>>()
    }

    pub fn get_entry_path(&self, entry_name: &str) -> String {
        let path = Path::new(&self.entry_dir).join(entry_name);
        path.to_string_lossy().to_string()
    }
}


pub fn initialize_database(path_config: &PathConfig) {
    let conn = Connection::open(&path_config.db).expect("Could not initialize database");

    conn.execute("
        CREATE TABLE IF NOT EXISTS entries (
            number INTEGER UNIQUE,
            name TEXT PRIMARY KEY NOT NULL,
            entry_date TEXT,
            access_date TEXT
        )
    ", ()).expect("Could not add table");

    let files = path_config.get_file();

    let mut entries = files.iter()
        .map(|f| Entry::from_file(&path_config.entry_dir, f))
        .collect::<Vec<Entry>>();

    // load into database

    sort_entries_by_number(&mut entries);

    for entry in entries {
        let entry_string = entry.entry_string();
        let access_string = entry.access_string();
        
        if let Some(number) = entry.number {
            conn.execute(
                "INSERT INTO entries (number, name, entry_date, access_date) VALUES (?1, ?2, ?3, ?4)", 
                (number, entry.name, entry_string, access_string)).expect("Could not add table");
        } else {
            conn.execute(
                "INSERT INTO entries (name, entry_date, access_date) VALUES (?1, ?2, ?3)", 
                (entry.name, entry_string, access_string)).expect("Could not add table");
        }
        
    }

    let mut stmt = conn.prepare("SELECT * FROM entries").expect("Entries not found");

    let entry_rows = stmt.query_map([], |row| {
        let entry = Entry::build_from_row(&path_config.entry_dir, &row);
        entry
    }).expect("Error reading rows");
    
    for row in entry_rows {
        println!("Found row: {:?}", row.unwrap());
    }
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
