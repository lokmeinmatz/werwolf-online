use std::sync::Mutex;
use std::sync::Arc;
use rusqlite::Connection;
use std::path::Path;
use log::{error};

pub struct Database(Arc<Mutex<Connection>>);


impl Database {
    pub fn open(path: &Path) -> Result<Database, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        Database::verify(&conn)?;
        Ok(Database(Arc::new(Mutex::new(conn))))
    }

    /// checks if the connected database contains needed tables and columns
    fn verify(conn: &Connection) -> Result<(), String> {

        let needed_tables = [
            ("sessions", vec!["id", "created", "active", "settings"])
        ];

        let mut table_check = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name = ?").map_err(|e| e.to_string())?;
        let mut col_check = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name = ?").map_err(|e| e.to_string())?;

        for (req_table, req_cols) in &needed_tables {
            if table_check.query(&[req_table])
                .map_err(|e| e.to_string())?
                .next().map_err(|e| e.to_string())?.is_none() {
                error!(target: "database", "Required table not found: {}", req_table);
                return Err(format!("Missing table: {}", req_table));
            }
        }


        Ok(())

    }
}