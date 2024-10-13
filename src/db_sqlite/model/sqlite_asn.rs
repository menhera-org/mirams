
use crate::db_sqlite::SqliteConnection;
use crate::types::{Error, ErrorKind};

use crate::asn::AsnAssignmentStore;

use r2d2_sqlite::rusqlite;

#[derive(Debug, Clone)]
pub struct SqliteAsnAssignmentStore {
    db: SqliteConnection,
}

impl SqliteAsnAssignmentStore {
    pub fn new(db: SqliteConnection) -> Self {
        SqliteAsnAssignmentStore { db }
    }
}

impl AsnAssignmentStore for SqliteAsnAssignmentStore {
    fn get_space(&self, space_id: i32) -> Result<crate::asn::AssignmentSpaceAsn, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, space_visibility, asn_from, asn_to FROM assignment_space_asn WHERE id = ?")?;
        let mut rows = stmt.query(rusqlite::params![space_id])?;
        let row = rows.next()?;
        let space = match row {
            Some(row) => {
                let space = crate::asn::AssignmentSpaceAsn {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    space_visibility: row.get(3)?,
                    asn_from: row.get(4)?,
                    asn_to: row.get(5)?,
                };
                Some(space)
            },
            None => None,
        };
        let space = if let Some(space) = space {
            space
        } else {
            return Err(Error::new(ErrorKind::NotFound,"Assignment space not found".to_string()));
        };
        Ok(space)
    }

    fn get_spaces(&self) -> Result<Vec<crate::asn::AssignmentSpaceAsn>, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, space_visibility, asn_from, asn_to FROM assignment_space_asn ORDER BY asn_from ASC")?;
        let rows = stmt.query_map(rusqlite::params![], |row| {
            Ok(crate::asn::AssignmentSpaceAsn {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                space_visibility: row.get(3)?,
                asn_from: row.get(4)?,
                asn_to: row.get(5)?,
            })
        })?;
        let mut spaces = Vec::new();
        for space in rows {
            spaces.push(space?);
        }
        Ok(spaces)
    }

    fn create_space(&self, space: &crate::asn::AssignmentSpaceAsn) -> Result<i32, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("INSERT INTO assignment_space_asn (name, description, space_visibility, asn_from, asn_to) VALUES (?, ?, ?, ?, ?)")?;
        stmt.execute(rusqlite::params![space.name, space.description, space.space_visibility as i32, space.asn_from, space.asn_to])?;
        Ok(conn.last_insert_rowid() as i32)
    }

    fn update_space(&self, id: i32, name: &str, description: &str) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("UPDATE assignment_space_asn SET name = ?, description = ? WHERE id = ?")?;
        stmt.execute(rusqlite::params![name, description, id])?;
        Ok(())
    }

    fn delete_space(&self, space_id: i32) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("DELETE FROM assignment_space_asn WHERE id = ?")?;
        stmt.execute(rusqlite::params![space_id])?;
        Ok(())
    }

    fn get_pool(&self, pool_id: i32) -> Result<crate::asn::AssignmentPoolAsn, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, pool_visibility, assignment_space_id, asn_from, asn_to FROM assignment_pool_asn WHERE id = ?")?;
        let mut rows = stmt.query(rusqlite::params![pool_id])?;
        let row = rows.next()?;
        let pool = match row {
            Some(row) => {
                let pool = crate::asn::AssignmentPoolAsn {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    pool_visibility: row.get(3)?,
                    assignment_space_id: row.get(4)?,
                    asn_from: row.get(5)?,
                    asn_to: row.get(6)?,
                };
                Some(pool)
            },
            None => None,
        };
        let pool = if let Some(pool) = pool {
            pool
        } else {
            return Err(Error::new(ErrorKind::NotFound,"Assignment pool not found".to_string()));
        };
        Ok(pool)
    }

    fn get_pools(&self, space_id: i32) -> Result<Vec<crate::asn::AssignmentPoolAsn>, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, pool_visibility, assignment_space_id, asn_from, asn_to FROM assignment_pool_asn WHERE assignment_space_id = ? ORDER BY asn_from ASC")?;
        let rows = stmt.query_map(rusqlite::params![space_id], |row| {
            Ok(crate::asn::AssignmentPoolAsn {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                pool_visibility: row.get(3)?,
                assignment_space_id: row.get(4)?,
                asn_from: row.get(5)?,
                asn_to: row.get(6)?,
            })
        })?;
        let mut pools = Vec::new();
        for pool in rows {
            pools.push(pool?);
        }
        Ok(pools)
    }

    fn create_pool(&self, pool: &crate::asn::AssignmentPoolAsn) -> Result<i32, Error> {
        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction()?;

        {
            // Check if the pool ASN range is within the space range
            let mut space_stmt = tx.prepare("SELECT asn_from, asn_to FROM assignment_space_asn WHERE id = ?")?;
            let space = space_stmt.query_row(rusqlite::params![pool.assignment_space_id], |row| {
                Ok((row.get::<_, u32>(0)?, row.get::<_, u32>(1)?))
            })?;

            if pool.asn_from < space.0 || pool.asn_to > space.1 {
                return Err(Error::new(ErrorKind::InvalidInput, "Pool ASN range is out of space range".to_string()));
            }
        }

        {
            // Check for overlapping pools
            let mut check_stmt = tx.prepare("SELECT COUNT(*) FROM assignment_pool_asn WHERE assignment_space_id = ? AND ((asn_from <= ? AND asn_to >= ?) OR (asn_from <= ? AND asn_to >= ?))")?;
            let count: i32 = check_stmt.query_row(rusqlite::params![pool.assignment_space_id, pool.asn_from, pool.asn_from, pool.asn_to, pool.asn_to], |row| row.get(0))?;

            if count > 0 {
                return Err(Error::new(ErrorKind::InvalidInput, "Overlapping assignment pool exists".to_string()));
            }
        }

        {
            // Insert the new pool
            let mut insert_stmt = tx.prepare("INSERT INTO assignment_pool_asn (name, description, pool_visibility, assignment_space_id, asn_from, asn_to) VALUES (?, ?, ?, ?, ?, ?)")?;
            insert_stmt.execute(rusqlite::params![pool.name, pool.description, pool.pool_visibility as i32, pool.assignment_space_id, pool.asn_from, pool.asn_to])?;
        }

        tx.commit()?;
        let id = conn.last_insert_rowid() as i32;
        Ok(id)
    }

    fn update_pool(&self, id: i32, name: &str, description: &str) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("UPDATE assignment_pool_asn SET name = ?, description = ? WHERE id = ?")?;
        stmt.execute(rusqlite::params![name, description, id])?;
        Ok(())
    }

    fn delete_pool(&self, pool_id: i32) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("DELETE FROM assignment_pool_asn WHERE id = ?")?;
        stmt.execute(rusqlite::params![pool_id])?;
        Ok(())
    }

    fn get_assignment(&self, assignment_id: i32) -> Result<crate::asn::AssignmentAsn, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, assignment_pool_id, asn FROM assignment_asn WHERE id = ?")?;
        let mut rows = stmt.query(rusqlite::params![assignment_id])?;
        let row = rows.next()?;
        let assignment = match row {
            Some(row) => {
                let assignment = crate::asn::AssignmentAsn {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    assignment_pool_id: row.get(3)?,
                    asn: row.get(4)?,
                };
                Some(assignment)
            },
            None => None,
        };
        let assignment = if let Some(assignment) = assignment {
            assignment
        } else {
            return Err(Error::new(ErrorKind::NotFound,"Assignment not found".to_string()));
        };
        Ok(assignment)
    }

    fn get_assignments(&self, pool_id: i32) -> Result<Vec<crate::asn::AssignmentAsn>, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, assignment_pool_id, asn FROM assignment_asn WHERE assignment_pool_id = ? ORDER BY asn ASC")?;
        let rows = stmt.query_map(rusqlite::params![pool_id], |row| {
            Ok(crate::asn::AssignmentAsn {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                assignment_pool_id: row.get(3)?,
                asn: row.get(4)?,
            })
        })?;
        let mut assignments = Vec::new();
        for assignment in rows {
            assignments.push(assignment?);
        }
        Ok(assignments)
    }

    fn create_assignment(&self, assignment: &crate::asn::AssignmentAsn) -> Result<i32, Error> {
        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction()?;

        {
            // Check for overlapping assignments within the same pool
            let mut check_stmt = tx.prepare("SELECT COUNT(*) FROM assignment_asn WHERE assignment_pool_id = ? AND asn = ?")?;
            let count: i32 = check_stmt.query_row(rusqlite::params![assignment.assignment_pool_id, assignment.asn], |row| row.get(0))?;

            if count > 0 {
                return Err(Error::new(ErrorKind::InvalidInput, "Overlapping assignment exists".to_string()));
            }
        }

        {
            // Check if the assignment ASN is within the pool range
            let mut pool_stmt = tx.prepare("SELECT asn_from, asn_to FROM assignment_pool_asn WHERE id = ?")?;
            let pool = pool_stmt.query_row(rusqlite::params![assignment.assignment_pool_id], |row| {
                Ok((row.get::<_, u32>(0)?, row.get::<_, u32>(1)?))
            })?;

            if assignment.asn < pool.0 || assignment.asn > pool.1 {
                return Err(Error::new(ErrorKind::InvalidInput, "Assignment ASN is out of pool range".to_string()));
            }
        }

        {
            // Insert the new assignment
            let mut insert_stmt = tx.prepare("INSERT INTO assignment_asn (name, description, assignment_pool_id, asn) VALUES (?, ?, ?, ?)")?;
            insert_stmt.execute(rusqlite::params![assignment.name, assignment.description, assignment.assignment_pool_id, assignment.asn])?;
        }

        tx.commit()?;
        let id = conn.last_insert_rowid() as i32;
        Ok(id)
    }

    fn update_assignment(&self, id: i32, name: &str, description: &str) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("UPDATE assignment_asn SET name = ?, description = ? WHERE id = ?")?;
        stmt.execute(rusqlite::params![name, description, id])?;
        Ok(())
    }

    fn delete_assignment(&self, assignment_id: i32) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("DELETE FROM assignment_asn WHERE id = ?")?;
        stmt.execute(rusqlite::params![assignment_id])?;
        Ok(())
    }
}

