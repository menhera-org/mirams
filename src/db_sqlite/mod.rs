mod migration;
pub mod model;
use migration::migrate_assignments;

#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod security_tests;

use r2d2_sqlite::rusqlite;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};

use crate::types::Error;
use crate::types::ErrorKind;
use crate::types::ErrorWithKind;

use crate::store::DbConnection;

pub use crate::types::ObjectVisibility;

// Schema versioning
const SCHEMA_VERSION: i32 = 3;

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
        let value: i32 = value
            .as_i64()?
            .try_into()
            .map_err(|_| FromSqlError::InvalidType)?;
        match value {
            0 => Ok(ObjectVisibility::Public),
            1 => Ok(ObjectVisibility::Private),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for ObjectVisibility {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok((*self as i64).into())
    }
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
    assignment_visibility INTEGER NOT NULL,
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
    assignment_visibility INTEGER NOT NULL,
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
    assignment_visibility INTEGER NOT NULL,
    asn UNSIGNED INTEGER NOT NULL,
    FOREIGN KEY (assignment_pool_id) REFERENCES assignment_pool_asn (id) ON DELETE CASCADE
);

CREATE INDEX assignment_asn_asn ON assignment_asn (asn);
"#;

// Repair duplicate accounts created by the old password setter, keeping the
// latest password and revoking every token associated with duplicate names.
const MIGRATION_2: &str = r#"
DELETE FROM api_key WHERE user_id IN (
    SELECT id FROM user WHERE name IN (
        SELECT name FROM user GROUP BY name HAVING COUNT(*) > 1
    )
);
DELETE FROM user WHERE id NOT IN (SELECT MAX(id) FROM user GROUP BY name);
DROP INDEX user_name;
CREATE UNIQUE INDEX user_name ON user (name);
"#;

// Actual code below

#[derive(Debug, Clone)]
pub struct SqliteConnection {
    pool: r2d2::Pool<SqliteConnectionManager>,
}

impl SqliteConnection {
    pub fn open_file(path: &str) -> Result<SqliteConnection, Error> {
        let manager = SqliteConnectionManager::file(path)
            .with_init(|conn| conn.execute_batch("PRAGMA foreign_keys=ON;"));
        let pool = r2d2::Pool::new(manager)?;
        let db = SqliteConnection { pool };
        db.initialize()?;
        Ok(db)
    }

    pub fn open_memory() -> Result<SqliteConnection, Error> {
        let manager = SqliteConnectionManager::memory()
            .with_init(|conn| conn.execute_batch("PRAGMA foreign_keys=ON;"));
        let pool = r2d2::Pool::new(manager)?;
        let db = SqliteConnection { pool };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<(), Error> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        tx.execute_batch(SCHEMA_VERSION_TABLE)?;
        let version = tx.query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            rusqlite::params![],
            |row| {
                let version: i32 = row.get(0)?;
                Ok(version)
            },
        );
        let mut version = match version {
            Ok(version) => version,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                tx.execute_batch(MIGRATION_1)?;
                tx.execute("INSERT INTO schema_version (id, version) VALUES (1, 1)", [])?;
                1
            }
            Err(error) => return Err(error.into()),
        };
        if !(1..=SCHEMA_VERSION).contains(&version) {
            return Err(Error::new(
                ErrorKind::InternalError,
                format!("Unsupported database schema version: {version}"),
            ));
        }
        if version == 1 {
            tx.execute_batch(MIGRATION_2)?;
            tx.execute("INSERT INTO schema_version (id, version) VALUES (2, 2)", [])?;
            version = 2;
        }
        if version == 2 {
            migrate_assignments(&tx)?;
            tx.execute("INSERT INTO schema_version (id, version) VALUES (3, 3)", [])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, Error> {
        Ok(self.pool.get()?)
    }
}

impl DbConnection for SqliteConnection {
    fn user_store(&self) -> Box<dyn crate::user::UserStore> {
        Box::new(model::SqliteUserStore::new(self.clone()))
    }

    fn assignment_store(&self) -> Box<dyn crate::assignment::AssignmentStore> {
        Box::new(model::SqliteAssignmentStore::new(self.clone()))
    }
}
