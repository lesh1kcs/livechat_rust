use rusqlite::{Connection, Result, params};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ChatDatabase {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub session_id: String,
    pub created_at: String,
}

impl ChatDatabase {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        let db = ChatDatabase {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_auth_table()?;

        println!("database initialized at: {}",db_path);
        Ok(db)
    }

    fn init_auth_table(&self) -> Result<()>{
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            session_id TEXT NOT NULL UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [], 
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_username ON users(username)",
            [],
        )?;

        println!("Auth table created");
        Ok(())
    }

    pub fn register_user(&self, username: &str, session_id: &str) -> Result<i64>{
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO users(username, session_id) VALUES (?1, ?2)",
            params![username, session_id],
        )?;

        let user_id = conn.last_insert_rowid();
        Ok(user_id)
    }

    pub fn username_exists(&self, username: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM users WHERE username =?1)", 
        params![username], 
        |row| row.get(0))?;
        Ok(exists)
    }

    pub fn update_session(&self, username: &str, new_session_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows_being_affected = conn.execute(
            "UPDATE users SET session_id = ?1 WHERE username = ?2",
            params![username, new_session_id],
        )?;

        if rows_being_affected > 0 {
            println!("Updated session for user '{}'", username);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_user_by_session(&self, session_id: &str) -> Result<Option<User>>{
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, username, session_id, created_at
            FROM users
            WHERE session_id = ?1"
        )?;

        let user = stmt.query_row(params![session_id], |row| {
            Ok(User{
                id: row.get(0)?,
                username: row.get(1)?,
                session_id: row.get(2)?,
                created_at: row.get(3)?,
            })
        }).ok();
        Ok(user)
    }

    pub fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, username, session_id, created_at
            FROM users
            WHERE username = ?1"
        )?;

        let user = stmt.query_row(params![username], |row|{
            Ok(User{
                id: row.get(0)?,
                username: row.get(1)?,
                session_id: row.get(2)?,
                created_at: row.get(3)?,
            })
        }).ok();
        Ok(user)
    }

    pub fn logout_user(&self, session_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows_being_affected = conn.execute(
            "DELETE FROM users WHERE session_id = ?1",
            params![session_id],
        )?;

        if rows_being_affected > 0 {
            println!("User logged out (session: {}", session_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub fn get_active_users(&self) -> Result<Vec<User>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, username, session_id, created_at FROM users")?;
        
        let users = stmt.query_map([],|row| {
            Ok(User{
                id: row.get(0)?,
                username: row.get(1)?,
                session_id: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
        Ok(users)
    }
}