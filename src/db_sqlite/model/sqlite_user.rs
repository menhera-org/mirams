
use crate::db_sqlite::SqliteConnection;
use crate::types::{Error, ErrorKind};

use crate::user::UserStore;

use r2d2_sqlite::rusqlite;

use crate::user::{
    hash_password,
    verify_password,
};


use argon2::password_hash::{
    rand_core::OsRng,
    rand_core::RngCore,
};


// Structs for tables

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub hashed_password: String,
}


#[derive(Debug, Clone)]
pub struct SqliteUserStore {
    db: SqliteConnection,
}

impl SqliteUserStore {
    pub fn new(db: SqliteConnection) -> Self {
        SqliteUserStore { db }
    }

    pub fn check_password(&self, username: &str, password: &str) -> Result<bool, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, hashed_password FROM user WHERE name = ?")?;
        let mut rows = stmt.query(rusqlite::params![username])?;
        let row = rows.next()?;
        let user = match row {
            Some(row) => {
                let user = User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    hashed_password: row.get(2)?,
                };
                Some(user)
            },
            None => None,
        };
        let user = if let Some(user) = user {
            user
        } else {
            return Err(Error::new(ErrorKind::NotFound,"User not found".to_string()));
        };
        let result = verify_password(&user.hashed_password, password).map_err(|_| Error::new(ErrorKind::InvalidInput,"Password verification failed".to_string()))?;
        Ok(result)
    }

    fn update_user(&self, name: &str, hashed_password: &str) -> Result<i32, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("INSERT OR REPLACE INTO user (name, hashed_password) VALUES (?, ?)")?;
        let id = stmt.insert(rusqlite::params![name, hashed_password])?;
        Ok((id & 0x7FFFFFFF) as i32)
    }

    /// Update the password for a user. If the user does not exist, it will be created.
    pub fn set_password(&self, username: &str, password: &str) -> Result<(), Error> {
        let hashed_password = hash_password(password).map_err(|_| Error::new(ErrorKind::InternalError,"Password hashing failed".to_string()))?;
        self.update_user(username, &hashed_password)?;
        Ok(())
    }

    pub fn delete_user(&self, username: &str) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("DELETE FROM user WHERE name = ?")?;
        stmt.execute(rusqlite::params![username])?;
        Ok(())
    }

    pub fn generate_api_key(&self, username: &str) -> Result<String, Error> {
        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction()?;
        let user = {
            let mut stmt = tx.prepare("SELECT id, name, hashed_password FROM user WHERE name = ?")?;
            let mut rows = stmt.query(rusqlite::params![username])?;
            let row = rows.next()?;
            let user = match row {
                Some(row) => {
                    let user = User {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        hashed_password: row.get(2)?,
                    };
                    Some(user)
                },
                None => None,
            };
            user
        };
        let user = if let Some(user) = user {
            user
        } else {
            return Err(Error::new(ErrorKind::NotFound,"User not found".to_string()));
        };
        let mut api_key = [0u8; 32];
        OsRng.fill_bytes(&mut api_key);
        let api_key = hex::encode(api_key);
        {
            let mut stmt = tx.prepare("INSERT OR REPLACE INTO api_key (key, user_id) VALUES (?, ?)")?;
            stmt.execute(rusqlite::params![&api_key, user.id])?;
        }
        tx.commit()?;
        Ok(api_key)
    }

    pub fn get_user_from_api_key(&self, api_key: &str) -> Result<Option<String>, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT user.id, user.name, user.hashed_password FROM user JOIN api_key ON user.id = api_key.user_id WHERE api_key.key = ?")?;
        let mut rows = stmt.query(rusqlite::params![api_key])?;
        let row = rows.next()?;
        let user = match row {
            Some(row) => {
                let user = User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    hashed_password: row.get(2)?,
                };
                Some(user)
            },
            None => None,
        };
        let username = if let Some(user) = user {
            Some(user.name)
        } else {
            None
        };
        Ok(username)
    }

    pub fn list_users(&self) -> Result<Vec<String>, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT name FROM user ORDER BY name ASC")?;
        let mut rows = stmt.query(rusqlite::params![])?;
        let mut users = Vec::new();
        while let Some(row) = rows.next()? {
            let name: String = row.get(0)?;
            users.push(name);
        }
        Ok(users)
    }
}

impl UserStore for SqliteUserStore {
    fn check_password(&self, username: &str, password: &str) -> Result<bool, Error> {
        SqliteUserStore::check_password(self, username, password)
    }

    fn set_password(&self, username: &str, password: &str) -> Result<(), Error> {
        SqliteUserStore::set_password(self, username, password)
    }

    fn delete_user(&self, username: &str) -> Result<(), Error> {
        SqliteUserStore::delete_user(self, username)
    }

    fn generate_api_key(&self, username: &str) -> Result<String, Error> {
        SqliteUserStore::generate_api_key(self, username)
    }

    fn get_user_from_api_key(&self, api_key: &str) -> Result<Option<String>, Error> {
        SqliteUserStore::get_user_from_api_key(self, api_key)
    }

    fn list_users(&self) -> Result<Vec<String>, Error> {
        SqliteUserStore::list_users(self)
    }
}
