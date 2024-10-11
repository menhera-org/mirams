
pub mod model;

use r2d2_sqlite::rusqlite;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};

use crate::ipv4;
use crate::ipv6;
use crate::types::ErrorKind;
use crate::types::ErrorWithKind;
use crate::types::Error;

use crate::store::DbConnection;

pub use crate::types::ObjectVisibility;


// Schema versioning
const SCHEMA_VERSION: i32 = 1;


// Structs for tables

impl ErrorWithKind for rusqlite::Error {
    fn kind(&self) -> ErrorKind {
        match self {
            rusqlite::Error::QueryReturnedNoRows => ErrorKind::NotFound,
            _ => ErrorKind::DatabaseError,
        }
    }
}

impl ErrorWithKind for r2d2::Error {
    fn kind(&self) -> ErrorKind {
        ErrorKind::DatabaseError
    }
}

impl FromSql for ObjectVisibility {
    fn column_result(value: ValueRef) -> Result<Self, FromSqlError> {
        let value: i32 = value.as_i64()?.try_into().map_err(|_| FromSqlError::InvalidType)?;
        match value {
            0 => Ok(ObjectVisibility::Public),
            1 => Ok(ObjectVisibility::Private),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for ObjectVisibility {
    fn to_sql(&self) -> Result<ToSqlOutput, rusqlite::Error> {
        Ok((*self as i64).into())
    }
}


// Users and API keys

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub hashed_password: String,
}

#[derive(Debug)]
pub struct ApiKey {
    pub id: i32,
    pub key: String,
    pub user_id: i32,
}

// IPv4 and IPv6 prefixes

#[derive(Debug)]
pub struct AssignmentSpaceIpv4 {
    pub id: i32,
    pub name: String,
    pub description: String, // Document actual usage ratio, purpose, etc.
    pub space_visibility: ObjectVisibility,
    pub ipv4_prefix: [u8; 4], // big-endian bit and byte order
    pub ipv4_prefix_len: i32,
}

#[derive(Debug)]
pub struct AssignmentPoolIpv4 {
    pub id: i32,
    pub assignment_space_id: i32,
    pub name: String,
    pub description: String, // Document actual usage ratio, purpose, etc.
    pub pool_visibility: ObjectVisibility,
    pub ipv4_prefix: [u8; 4], // big-endian bit and byte order
    pub ipv4_prefix_len: i32,
}

#[derive(Debug)]
pub struct AssignmentIpv4 {
    pub id: i32,
    pub assignment_pool_id: i32,
    pub name: String,
    pub description: String, // Document actual usage ratio, purpose, etc.
    pub ipv4_prefix: [u8; 4], // big-endian bit and byte order
    pub ipv4_prefix_len: i32,
}

#[derive(Debug)]
pub struct AssignmentSpaceIpv6 {
    pub id: i32,
    pub name: String,
    pub description: String, // Document actual usage ratio, purpose, etc.
    pub space_visibility: ObjectVisibility,
    pub ipv6_prefix: [u8; 16], // big-endian bit and byte order
    pub ipv6_prefix_len: i32,
}

#[derive(Debug)]
pub struct AssignmentPoolIpv6 {
    pub id: i32,
    pub assignment_space_id: i32,
    pub name: String,
    pub description: String, // Document actual usage ratio, purpose, etc.
    pub pool_visibility: ObjectVisibility,
    pub ipv6_prefix: [u8; 16], // big-endian bit and byte order
    pub ipv6_prefix_len: i32,
}

#[derive(Debug)]
pub struct AssignmentIpv6 {
    pub id: i32,
    pub assignment_pool_id: i32,
    pub name: String,
    pub description: String, // Document actual usage ratio, purpose, etc.
    pub ipv6_prefix: [u8; 16], // big-endian bit and byte order
    pub ipv6_prefix_len: i32,
}

// ASNs (usually private)
// We allow 32-bit ASNs for simplicity.

#[derive(Debug)]
pub struct AssignmentSpaceAsn {
    pub id: i32,
    pub name: String,
    pub description: String, // Document actual usage ratio, purpose, etc.
    pub space_visibility: ObjectVisibility,
    pub asn_from: i32,
    pub asn_to: i32,
}

#[derive(Debug)]
pub struct AssignmentPoolAsn {
    pub id: i32,
    pub assignment_space_id: i32,
    pub name: String,
    pub description: String, // Document actual usage ratio, purpose, etc.
    pub pool_visibility: ObjectVisibility,
    pub asn_from: i32,
    pub asn_to: i32,
}

#[derive(Debug)]
pub struct AssignmentAsn {
    pub id: i32,
    pub assignment_pool_id: i32,
    pub name: String,
    pub description: String, // Document purpose, etc.
    pub asn: i32,
}


// SQL statements
// We use BLOB for text fields to avoid quirks with UTF-8 encoding and collation.

const SCHEMA_VERSION_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    id INTEGER PRIMARY KEY,
    version INTEGER NOT NULL UNIQUE
);
"#;

const MIGRATION_1: &str = r#"
CREATE TABLE user (
    id INTEGER PRIMARY KEY,
    name BLOB NOT NULL,
    hashed_password TEXT NOT NULL
);

CREATE INDEX user_name ON user (name);

CREATE TABLE api_key (
    id INTEGER PRIMARY KEY,
    key BLOB NOT NULL,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user (id) ON DELETE CASCADE
);

CREATE TABLE assignment_space_ipv4 (
    id INTEGER PRIMARY KEY,
    name BLOB NOT NULL,
    description BLOB NOT NULL,
    space_visibility INTEGER NOT NULL,
    ipv4_prefix BLOB NOT NULL CHECK(length(ipv4_prefix) = 4),
    ipv4_prefix_len INTEGER NOT NULL CHECK(ipv4_prefix_len BETWEEN 0 AND 32)
);

CREATE INDEX assignment_space_ipv4_prefix ON assignment_space_ipv4 (ipv4_prefix);
CREATE INDEX assignment_space_ipv4_prefix_len ON assignment_space_ipv4 (ipv4_prefix_len);

CREATE TABLE assignment_pool_ipv4 (
    id INTEGER PRIMARY KEY,
    assignment_space_id INTEGER NOT NULL,
    name BLOB NOT NULL,
    description BLOB NOT NULL,
    pool_visibility INTEGER NOT NULL,
    ipv4_prefix BLOB NOT NULL CHECK(length(ipv4_prefix) = 4),
    ipv4_prefix_len INTEGER NOT NULL CHECK(ipv4_prefix_len BETWEEN 0 AND 32),
    FOREIGN KEY (assignment_space_id) REFERENCES assignment_space_ipv4 (id) ON DELETE CASCADE
);

CREATE INDEX assignment_pool_ipv4_prefix ON assignment_pool_ipv4 (ipv4_prefix);
CREATE INDEX assignment_pool_ipv4_prefix_len ON assignment_pool_ipv4 (ipv4_prefix_len);

CREATE TABLE assignment_ipv4 (
    id INTEGER PRIMARY KEY,
    assignment_pool_id INTEGER NOT NULL,
    name BLOB NOT NULL,
    description BLOB NOT NULL,
    ipv4_prefix BLOB NOT NULL CHECK(length(ipv4_prefix) = 4),
    ipv4_prefix_len INTEGER NOT NULL CHECK(ipv4_prefix_len BETWEEN 0 AND 32),
    FOREIGN KEY (assignment_pool_id) REFERENCES assignment_pool_ipv4 (id) ON DELETE CASCADE
);

CREATE INDEX assignment_ipv4_prefix ON assignment_ipv4 (ipv4_prefix);
CREATE INDEX assignment_ipv4_prefix_len ON assignment_ipv4 (ipv4_prefix_len);

CREATE TABLE assignment_space_ipv6 (
    id INTEGER PRIMARY KEY,
    name BLOB NOT NULL,
    description BLOB NOT NULL,
    space_visibility INTEGER NOT NULL,
    ipv6_prefix BLOB NOT NULL CHECK(length(ipv6_prefix) = 16),
    ipv6_prefix_len INTEGER NOT NULL CHECK(ipv6_prefix_len BETWEEN 0 AND 128)
);

CREATE INDEX assignment_space_ipv6_prefix ON assignment_space_ipv6 (ipv6_prefix);
CREATE INDEX assignment_space_ipv6_prefix_len ON assignment_space_ipv6 (ipv6_prefix_len);

CREATE TABLE assignment_pool_ipv6 (
    id INTEGER PRIMARY KEY,
    assignment_space_id INTEGER NOT NULL,
    name BLOB NOT NULL,
    description BLOB NOT NULL,
    pool_visibility INTEGER NOT NULL,
    ipv6_prefix BLOB NOT NULL CHECK(length(ipv6_prefix) = 16),
    ipv6_prefix_len INTEGER NOT NULL CHECK(ipv6_prefix_len BETWEEN 0 AND 128),
    FOREIGN KEY (assignment_space_id) REFERENCES assignment_space_ipv6 (id) ON DELETE CASCADE
);

CREATE INDEX assignment_pool_ipv6_prefix ON assignment_pool_ipv6 (ipv6_prefix);
CREATE INDEX assignment_pool_ipv6_prefix_len ON assignment_pool_ipv6 (ipv6_prefix_len);

CREATE TABLE assignment_ipv6 (
    id INTEGER PRIMARY KEY,
    assignment_pool_id INTEGER NOT NULL,
    name BLOB NOT NULL,
    description BLOB NOT NULL,
    ipv6_prefix BLOB NOT NULL CHECK(length(ipv6_prefix) = 16),
    ipv6_prefix_len INTEGER NOT NULL CHECK(ipv6_prefix_len BETWEEN 0 AND 128),
    FOREIGN KEY (assignment_pool_id) REFERENCES assignment_pool_ipv6 (id) ON DELETE CASCADE
);

CREATE INDEX assignment_ipv6_prefix ON assignment_ipv6 (ipv6_prefix);
CREATE INDEX assignment_ipv6_prefix_len ON assignment_ipv6 (ipv6_prefix_len);

CREATE TABLE assignment_space_asn (
    id INTEGER PRIMARY KEY,
    name BLOB NOT NULL,
    description BLOB NOT NULL,
    space_visibility INTEGER NOT NULL,
    asn_from UNSIGNED INTEGER NOT NULL,
    asn_to UNSIGNED INTEGER NOT NULL,
    CHECK(asn_from <= asn_to)
);

CREATE INDEX assignment_space_asn_asn_from ON assignment_space_asn (asn_from);
CREATE INDEX assignment_space_asn_asn_to ON assignment_space_asn (asn_to);

CREATE TABLE assignment_pool_asn (
    id INTEGER PRIMARY KEY,
    assignment_space_id INTEGER NOT NULL,
    name BLOB NOT NULL,
    description BLOB NOT NULL,
    pool_visibility INTEGER NOT NULL,
    asn_from UNSIGNED INTEGER NOT NULL,
    asn_to UNSIGNED INTEGER NOT NULL,
    CHECK(asn_from <= asn_to),
    FOREIGN KEY (assignment_space_id) REFERENCES assignment_space_asn (id) ON DELETE CASCADE
);

CREATE INDEX assignment_pool_asn_asn_from ON assignment_pool_asn (asn_from);
CREATE INDEX assignment_pool_asn_asn_to ON assignment_pool_asn (asn_to);

CREATE TABLE assignment_asn (
    id INTEGER PRIMARY KEY,
    assignment_pool_id INTEGER NOT NULL,
    name BLOB NOT NULL,
    description BLOB NOT NULL,
    asn UNSIGNED INTEGER NOT NULL,
    FOREIGN KEY (assignment_pool_id) REFERENCES assignment_pool_asn (id) ON DELETE CASCADE
);

CREATE INDEX assignment_asn_asn ON assignment_asn (asn);
"#;


// Actual code below

#[derive(Debug, Clone)]
pub struct SqliteConnection {
    pool: r2d2::Pool<SqliteConnectionManager>,
}

impl SqliteConnection {
    pub fn open_file(path: &str) -> Result<SqliteConnection, Error> {
        let manager = SqliteConnectionManager::file(path);
        let pool = r2d2::Pool::new(manager)?;
        let db = SqliteConnection { pool };
        db.initialize()?;
        Ok(db)
    }

    pub fn open_memory() -> Result<SqliteConnection, Error> {
        let manager = SqliteConnectionManager::memory();
        let pool = r2d2::Pool::new(manager)?;
        let db = SqliteConnection { pool };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<(), Error> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        tx.execute_batch(SCHEMA_VERSION_TABLE)?;
        let version = tx.query_row("SELECT version FROM schema_version ORDER BY version DESC LIMIT 1", rusqlite::params![], |row| {
            let version: i32 = row.get(0)?;
            Ok(version)
        });
        match version {
            Ok(version) => {
                if version != SCHEMA_VERSION {
                    return Err(Error::new(ErrorKind::InternalError, format!("Database schema version mismatch: expected {}, got {}", SCHEMA_VERSION, version)));
                }
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                tx.execute("INSERT INTO schema_version (id, version) VALUES (1, ?)", [SCHEMA_VERSION])?;
                tx.execute_batch(MIGRATION_1)?;
            },
            Err(error) => return Err(error.into()),
        }
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn get_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, Error> {
        Ok(self.pool.get()?)
    }

    pub fn get_assignment_space_asn(&self, id: i32) -> Result<Option<AssignmentSpaceAsn>, Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, name, description, space_visibility, asn_from, asn_to FROM assignment_space_asn WHERE id = ?")?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        let row = rows.next()?;
        match row {
            Some(row) => {
                let assignment_space = AssignmentSpaceAsn {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    space_visibility: row.get(3)?,
                    asn_from: row.get(4)?,
                    asn_to: row.get(5)?,
                };
                Ok(Some(assignment_space))
            },
            None => Ok(None),
        }
    }

    pub fn get_assignment_pool_asn(&self, id: i32) -> Result<Option<AssignmentPoolAsn>, Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, assignment_space_id, name, description, pool_visibility, asn_from, asn_to FROM assignment_pool_asn WHERE id = ?")?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        let row = rows.next()?;
        match row {
            Some(row) => {
                let assignment_pool = AssignmentPoolAsn {
                    id: row.get(0)?,
                    assignment_space_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    pool_visibility: row.get(4)?,
                    asn_from: row.get(5)?,
                    asn_to: row.get(6)?,
                };
                Ok(Some(assignment_pool))
            },
            None => Ok(None),
        }
    }

    pub fn get_assignment_asn(&self, id: i32) -> Result<Option<AssignmentAsn>, Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, assignment_pool_id, name, description, asn FROM assignment_asn WHERE id = ?")?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        let row = rows.next()?;
        match row {
            Some(row) => {
                let assignment = AssignmentAsn {
                    id: row.get(0)?,
                    assignment_pool_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    asn: row.get(4)?,
                };
                Ok(Some(assignment))
            },
            None => Ok(None),
        }
    }

    pub fn create_assignment_space_asn(&self, name: &str, description: &str, space_visibility: ObjectVisibility, asn_from: u32, asn_to: u32) -> Result<i32, Error> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        if asn_from > asn_to {
            return Err(Error::new(ErrorKind::Other, "Invalid ASN range".to_string()));
        }
        {
            let mut stmt = tx.prepare("SELECT id FROM assignment_space_asn WHERE asn_from <= ? AND asn_to >= ?")?;
            let mut rows = stmt.query(rusqlite::params![asn_to, asn_from])?;
            let row = rows.next()?;
            if row.is_some() {
                return Err(Error::new(ErrorKind::Other, "Overlapping ASN range".to_string()));
            }
        }
        let id = {
            let mut stmt = tx.prepare("INSERT OR FAIL INTO assignment_space_asn (name, description, space_visibility, asn_from, asn_to) VALUES (?, ?, ?, ?, ?)")?;
            let id = stmt.insert(rusqlite::params![name, description, space_visibility, asn_from, asn_to])?;
            id
        };
        tx.commit()?;
        Ok((id & 0x7FFFFFFF) as i32)
    }

    pub fn update_assignment_space_asn(&self, id: i32, name: &str, description: &str, space_visibility: ObjectVisibility) -> Result<(), Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("UPDATE assignment_space_asn SET name = ?, description = ?, space_visibility = ? WHERE id = ?")?;
        stmt.execute(rusqlite::params![name, description, space_visibility, id])?;
        Ok(())
    }

    pub fn create_assignment_pool_asn(&self, assignment_space_id: i32, name: &str, description: &str, pool_visibility: ObjectVisibility, asn_from: u32, asn_to: u32) -> Result<i32, Error> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        if asn_from > asn_to {
            return Err(Error::new(ErrorKind::Other, "Invalid ASN range".to_string()));
        }
        {
            let mut stmt = tx.prepare("SELECT asn_from, asn_to FROM assignment_space_asn WHERE id = ?")?;
            let mut rows = stmt.query(rusqlite::params![assignment_space_id])?;
            let row = rows.next()?;
            match row {
                Some(row) => {
                    let space_asn_from: u32 = row.get(0)?;
                    let space_asn_to: u32 = row.get(1)?;
                    if asn_from < space_asn_from || asn_to > space_asn_to {
                        return Err(Error::new(ErrorKind::Other, "Pool ASN range not within space ASN range".to_string()));
                    }
                },
                None => return Err(Error::new(ErrorKind::Other, "Assignment space not found".to_string())),
            }
        }
        {
            let mut stmt = tx.prepare("SELECT id FROM assignment_pool_asn WHERE asn_from <= ? AND asn_to >= ?")?;
            let mut rows = stmt.query(rusqlite::params![asn_to, asn_from])?;
            let row = rows.next()?;
            if row.is_some() {
                return Err(Error::new(ErrorKind::Other, "Overlapping ASN range".to_string()));
            }
        }
        let id = {
            let mut stmt = tx.prepare("INSERT OR FAIL INTO assignment_pool_asn (assignment_space_id, name, description, pool_visibility, asn_from, asn_to) VALUES (?, ?, ?, ?, ?, ?)")?;
            let id = stmt.insert(rusqlite::params![assignment_space_id, name, description, pool_visibility, asn_from, asn_to])?;
            id
        };
        tx.commit()?;
        Ok((id & 0x7FFFFFFF) as i32)
    }

    pub fn update_assignment_pool_asn(&self, id: i32, name: &str, description: &str, pool_visibility: ObjectVisibility) -> Result<(), Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("UPDATE assignment_pool_asn SET name = ?, description = ?, pool_visibility = ? WHERE id = ?")?;
        stmt.execute(rusqlite::params![name, description, pool_visibility, id])?;
        Ok(())
    }

    pub fn create_assignment_asn(&self, assignment_pool_id: i32, name: &str, description: &str, asn: u32) -> Result<i32, Error> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare("SELECT asn_from, asn_to FROM assignment_pool_asn WHERE id = ?")?;
            let mut rows = stmt.query(rusqlite::params![assignment_pool_id])?;
            let row = rows.next()?;
            match row {
                Some(row) => {
                    let pool_asn_from: u32 = row.get(0)?;
                    let pool_asn_to: u32 = row.get(1)?;
                    if asn < pool_asn_from || asn > pool_asn_to {
                        return Err(Error::new(ErrorKind::Other, "Assignment ASN not within pool ASN range".to_string()));
                    }
                },
                None => return Err(Error::new(ErrorKind::Other, "Assignment pool not found".to_string())),
            }
        }
        {
            let mut stmt = tx.prepare("SELECT id FROM assignment_asn WHERE asn = ?")?;
            let mut rows = stmt.query(rusqlite::params![asn])?;
            let row = rows.next()?;
            if row.is_some() {
                return Err(Error::new(ErrorKind::Other, "Duplicate ASN".to_string()));
            }
        }
        let id = {
            let mut stmt = tx.prepare("INSERT OR FAIL INTO assignment_asn (assignment_pool_id, name, description, asn) VALUES (?, ?, ?, ?)")?;
            let id = stmt.insert(rusqlite::params![assignment_pool_id, name, description, asn])?;
            id
        };
        tx.commit()?;
        Ok((id & 0x7FFFFFFF) as i32)
    }

    pub fn update_assignment_asn(&self, id: i32, name: &str, description: &str) -> Result<(), Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("UPDATE assignment_asn SET name = ?, description = ? WHERE id = ?")?;
        stmt.execute(rusqlite::params![name, description, id])?;
        Ok(())
    }

    pub fn delete_assignment_space_asn(&self, id: i32) -> Result<(), Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("DELETE FROM assignment_space_asn WHERE id = ?")?;
        stmt.execute(rusqlite::params![id])?;
        Ok(())
    }

    pub fn delete_assignment_pool_asn(&self, id: i32) -> Result<(), Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("DELETE FROM assignment_pool_asn WHERE id = ?")?;
        stmt.execute(rusqlite::params![id])?;
        Ok(())
    }

    pub fn delete_assignment_asn(&self, id: i32) -> Result<(), Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("DELETE FROM assignment_asn WHERE id = ?")?;
        stmt.execute(rusqlite::params![id])?;
        Ok(())
    }
    
}

impl DbConnection for SqliteConnection {
    fn user_store(&self) -> Box<dyn crate::user::UserStore> {
        Box::new(model::SqliteUserStore::new(self.clone()))
    }

    fn ipv4_assignment_store(&self) -> Box<dyn ipv4::Ipv4AssignmentStore> {
        Box::new(model::SqliteIpv4AssignmentStore::new(self.clone()))
    }

    fn ipv6_assignment_store(&self) -> Box<dyn ipv6::Ipv6AssignmentStore> {
        Box::new(model::SqliteIpv6AssignmentStore::new(self.clone()))
    }

    fn asn_assignment_store(&self) -> Box<dyn crate::asn::AsnAssignmentStore> {
        Box::new(model::SqliteAsnAssignmentStore::new(self.clone()))
    }
}
