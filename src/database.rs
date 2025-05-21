use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Write},
    sync::Mutex,
};

pub struct Database {
    data: HashMap<String, ServerData>,
}

impl Database {
    fn load_from_file(path: &str) -> HashMap<String, ServerData> {
        // ensure file exists
        if File::open(path).is_err() {
            if let Ok(mut file) = File::create(path) {
                let _ = file.write_all(b"{}");
            }
        }

        let file = File::open(path);
        if file.is_err() {
            return HashMap::new();
        }

        let reader = BufReader::new(file.unwrap());

        match from_reader(reader) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to load database from file: {}, path: {}", e, path);
                // Attempt to create an empty database file if deserialization fails
                if let Err(create_err) = File::create(path) {
                    eprintln!("Failed to create a new database file: {}", create_err);
                } else {
                    eprintln!("Created a new empty database file because deserialization failed.");
                }
                HashMap::new()
            }
        }
    }

    fn save_to_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let db = DATABASE.lock()?;
        let mut file = File::create(path)?;
        let json = serde_json::to_string(&db.data)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn set_announcement_channel(
        server_id: String,
        channel_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            let db = DATABASE.lock();
            if db.is_err() {
                return Err("Failed to lock database".into());
            }
            let data = &mut db.unwrap().data;
            if data.get(&server_id).is_none() {
                data.insert(server_id.clone(), ServerData::empty());
            }
            data.get_mut(&server_id).unwrap().announcement_channel = channel_id;
        }
        Self::save_to_file("./database.json")?;
        Ok(())
    }

    pub fn get_data() -> Result<HashMap<String, ServerData>, Box<dyn std::error::Error>> {
        let db = DATABASE.lock()?;
        Ok(db.data.clone())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerData {
    pub announcement_channel: Option<String>,
}
impl ServerData {
    pub fn empty() -> Self {
        Self {
            announcement_channel: None,
        }
    }
}

static DATABASE: Lazy<Mutex<Database>> = Lazy::new(|| {
    Mutex::new(Database {
        data: Database::load_from_file("./database.json"),
    })
});
