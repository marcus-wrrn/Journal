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

    pub fn get_files(&self) -> Vec<String> {
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

pub struct EntryDB {
    pub conn: Connection,
    pub config: PathConfig
}

impl EntryDB {
    pub fn new(config: PathConfig) -> Self {
        Self {
            conn: Connection::open(&config.db).expect("Could not open Database"),
            config
        }
    }

    pub fn init_tables(&self) {
        self.conn.execute("
            CREATE TABLE IF NOT EXISTS entries (
                number INTEGER UNIQUE,
                name TEXT PRIMARY KEY NOT NULL,
                entry_date TEXT,
                access_date TEXT
            )
        ", ()).expect("Could not add table");
    
        self.conn.execute("
            CREATE TABLE IF NOT EXISTS tags (
                name TEXT PRIMARY KEY
            )
        ", ()).expect("Could not add tags table");
    
        self.conn.execute("
            CREATE TABLE IF NOT EXISTS entry_tags (
                tag TEXT,
                entry TEXT,
                PRIMARY KEY (entry, tag),
                FOREIGN KEY (tag) REFERENCES tags (name) ON DELETE CASCADE,
                FOREIGN KEY (entry) REFERENCES entries (name) ON DELETE CASCADE
            )
        ", ()).expect("Could not add tag reference table");
    }

    pub fn rebuild_database(&self) {
        self.init_tables();
    
        let files = self.config.get_files();
    
        let mut entries = files.iter()
            .map(|f| Entry::from_file(&self.config.entry_dir, f))
            .collect::<Vec<Entry>>();
    
        // load into database
    
        sort_entries_by_number(&mut entries);
    
        for entry in entries {
            let entry_string = entry.entry_string();
            let access_string = entry.access_string();
            
            if let Some(number) = entry.number {
                self.conn.execute(
                    "INSERT INTO entries (number, name, entry_date, access_date) VALUES (?1, ?2, ?3, ?4)", 
                    (number, entry.name, entry_string, access_string)).expect("Could not add table");
            } else {
                self.conn.execute(
                    "INSERT INTO entries (name, entry_date, access_date) VALUES (?1, ?2, ?3)", 
                    (entry.name, entry_string, access_string)).expect("Could not add table");
            }
            
        }
    
        let mut stmt = self.conn.prepare("SELECT * FROM entries").expect("Entries not found");
    
        let entry_rows = stmt.query_map([], |row| {
            let entry = Entry::build_from_row(&self.config.entry_dir, &row);
            entry
        }).expect("Error reading rows");
        
        for row in entry_rows {
            println!("Found row: {:?}", row.unwrap());
        }
    }

    pub fn update_entry_access_date(&self, entry: &mut Entry) -> &Self {
        entry.access_date = Some(chrono::offset::Local::now().into());
        if let Some(date) = entry.access_date {
            self.conn.execute(
                "UPDATE entries SET access_date = ?1 WHERE name = ?2", 
                (date.to_rfc2822(), entry.name.clone()))
                .expect("Could not update access date in DB");
        }
        self
    }

    pub fn get_entries(&self) -> Vec<Entry> {
        let mut stmt = self.conn.prepare("SELECT * FROM entries").expect("Could not select entries in DB");
        let entries = stmt.query_map([], |row| {
            Entry::build_from_row(&self.config.entry_dir, &row)
        }).expect("Error reading entries");

        entries.into_iter()
                .filter_map(|val| val.ok())
                .collect::<Vec<Entry>>()
    }
    //pub fn get_entries(&self)

    pub fn delete_entry(&self, entry: &mut Entry) {
        self.conn.execute(
            "DELETE FROM entries WHERE name = ?1", 
            (entry.name.clone(),))
            .expect("Could not delete entry from DB");

        fs::remove_file(&entry.path).expect("Could not delete file");
    }

    pub fn add_entry_to_db(&self, entry: &Entry) {
        if let Some(number) = entry.number {
            self.conn.execute(
                "INSERT INTO entries (number, name, entry_date, access_date) VALUES (?1, ?2, ?3, ?4)", 
                (number, entry.name.clone(), entry.entry_string(), entry.access_string())).expect("Could not add entry to DB");
        } else {
            self.conn.execute(
                "INSERT INTO entries (name, entry_date, access_date) VALUES (?1, ?2, ?3)", 
                (entry.name.clone(), entry.entry_string(), entry.access_string())).expect("Could not add entry to DB");
        }
    }

    pub fn create_custom_entry(&self, entry_name: &str) -> Entry {
        let entry = Entry::create_custom(&self.config, entry_name);
        entry.initialize();
        self.add_entry_to_db(&entry);
        entry
    }

    fn get_largest_entry_num(&self) -> u32 {
        let mut stmt = self.conn.prepare("SELECT MAX(number) FROM entries").expect("Entries not found");
        let largest_number: Option<u32> = stmt.query_row([], |row| row.get(0)).ok();
        if let Some(number) = largest_number {
            return number + 1;
        }
        1
    }

    pub fn create_default_entry(&self) -> Entry {
        let number = self.get_largest_entry_num();
        let entry = Entry::create_default(number, &self.config);
        entry.initialize();
        self.add_entry_to_db(&entry);
        entry
    }

    pub fn add_tag(&self, tag_name: &str) -> Result<()> {
        self.conn.execute("INSERT INTO tags (name) VALUES (?1)", (tag_name,))?;
        Ok(())
    }

    fn tag_exists(&self, tag: &str) -> bool {
        let mut stmt = self.conn.prepare("SELECT name FROM tags WHERE name = ?1").expect("Could not prepare tag check statement");
        let val = stmt.query((tag,));
        if let Ok(_) = val {
            return true;
        }
        false

    }

    pub fn assign_tag(&self, entry: &Entry, tag: &str) -> Result<()> {
        if !self.tag_exists(tag) {
            self.add_tag(tag).expect("Could not add tag");
        }
        self.conn.execute(
            "INSERT INTO entry_tags (tag, entry) VALUES (?1, ?2)", 
            (tag, &entry.name))?;
        
        Ok(())
    }

    pub fn change_name(&self, entry: &mut Entry, new_name: &str) {
        // Check if name already exists


        self.conn.execute(
            "UPDATE entries SET name = ?1 WHERE name = ?2", 
            (new_name, &entry.name)).expect("Could not update name");
        
        // Update filepath
        let path = self.config.get_entry_path(&entry.name);
        let new_path = self.config.get_entry_path(&new_name);
        std::fs::rename(path, new_path).expect("Could not rename filepath");
        
        // Change the name of the entry
        entry.name = new_name.to_string();

        // get entry path and rename the file

    }
}

