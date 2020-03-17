use crate::api::auth::SessionID;
use crate::api::net_types::PlayerData;
use crate::SessionData;
use log::{error, info};
use rusqlite::{params, Connection, Row, NO_PARAMS};
use std::path::Path;
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};

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
            ("sessions", vec!["id", "created", "active", "settings"]),
            (
                "users",
                vec!["session_id", "username", "role", "joined", "state"],
            ),
        ];

        let mut table_check = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name = ?")
            .map_err(|e| e.to_string())?;
        let mut col_check = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name = ?")
            .map_err(|e| e.to_string())?;

        for (req_table, req_cols) in &needed_tables {
            if table_check
                .query(&[req_table])
                .map_err(|e| e.to_string())?
                .next()
                .map_err(|e| e.to_string())?
                .is_none()
            {
                error!(target: "database", "Required table not found: {}", req_table);
                return Err(format!("Missing table: {}", req_table));
            }
        }

        Ok(())
    }

    pub fn get_locked_conn(&self) -> MutexGuard<Connection> {
        self.0.lock().unwrap()
    }

    pub fn get_session_data(conn: &mut Connection, sid: &SessionID) -> Option<SessionData> {
        use std::time;

        conn.query_row(
            "SELECT created, active, settings FROM sessions WHERE id = ?",
            &[sid.as_str()],
            |row| {
                Ok(SessionData {
                    id: sid.clone(),
                    created: time::UNIX_EPOCH
                        + time::Duration::from_secs(row.get::<usize, i64>(0)? as u64),
                    active: row.get(1)?,
                    settings: row.get(2)?,
                })
            },
        )
        .ok()
    }

    pub fn get_all_sessions<T: Sized>(
        conn: &mut Connection,
        extractor: fn(&Row) -> rusqlite::Result<T>,
    ) -> Vec<T> {
        let mut prep = conn
            .prepare("SELECT id, created, active FROM sessions")
            .unwrap();

        prep.query_map(NO_PARAMS, extractor)
            .unwrap()
            .filter_map(Result::ok)
            .collect()
    }

    pub fn get_sessions_active(conn: MutexGuard<Connection>) -> u32 {
        conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE active = 1",
            NO_PARAMS,
            |row| row.get(0),
        )
        .unwrap()
    }

    /// adds the player if the following conditions are met:
    /// 1) the session provided already exists
    /// 2) there's no player with the same name in that session
    /// 3) TODO: The player is not blacklisted by IP / Name
    ///
    /// Returns the player ID if created
    pub fn maybe_add_player(
        conn: &mut Connection,
        name: &str,
        sid: &SessionID,
    ) -> Result<(), String> {
        // 1) check if session exists
        let session_check: Result<(bool, Option<String>), _> = conn.query_row(
            "SELECT active, settings FROM sessions WHERE id = ?",
            &[sid.as_str()],
            |row| Ok((row.get_unwrap(0), row.get_unwrap(1))),
        );

        match session_check {
            Ok((true, _settings)) => {}
            Ok((false, _settings)) => {
                error!("Tried to add player to inactive session");
                return Err("Session inactive".into());
            }
            _ => {
                error!("Session doesn't exist");
                return Err("Session doesn't exist".into());
            }
        }

        // 2) no player with same name

        match conn.query_row(
            " SELECT * FROM users WHERE user_name = ? AND session_id IN (SELECT id FROM sessions \
            WHERE active = 1);",
            &[&name],
            |row| Ok(true),
        ) {
            Ok(true) => {
                error!("Two users with the same name tried to join TODO handle authorized rejoin");
                return Err("A user with the same name exists".into());
            }
            _ => {}
        }

        let joined = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64;

        // add player
        conn.execute(
            "INSERT INTO users (user_name, session_id, joined, state) VALUES (?, ?, ?, ?)",
            params![&name, sid.as_str(), joined, "waiting"],
        )
        .unwrap();

        Ok(())
    }

    pub fn get_players(conn: &mut Connection, sid: &SessionID) -> Vec<PlayerData> {
        let mut stmt = conn
            .prepare("SELECT user_name, role, joined, state FROM users WHERE session_id = ?")
            .unwrap();

        stmt.query_map(&[sid.as_str()], |usr_row| {
            Ok(PlayerData {
                name: usr_row.get_unwrap(0),
                role: usr_row.get_unwrap(1),
                joined: usr_row.get_unwrap::<usize, i64>(2) as u64,
                state: usr_row.get_unwrap(3),
            })
        })
        .unwrap()
        .filter_map(|e| e.ok())
        .collect()
    }
}
