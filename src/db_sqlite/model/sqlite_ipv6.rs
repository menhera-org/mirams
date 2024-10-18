
use crate::db_sqlite::SqliteConnection;
use crate::types::{Error, ErrorKind};

use crate::ipv6::Ipv6AssignmentStore;

use r2d2_sqlite::rusqlite;

#[derive(Debug, Clone)]
pub struct SqliteIpv6AssignmentStore {
    db: SqliteConnection,
}

impl SqliteIpv6AssignmentStore {
    pub fn new(db: SqliteConnection) -> Self {
        SqliteIpv6AssignmentStore { db }
    }
}

impl Ipv6AssignmentStore for SqliteIpv6AssignmentStore {
    fn get_space(&self, space_id: i32) -> Result<crate::ipv6::AssignmentSpaceIpv6, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, space_visibility, ipv6_prefix, ipv6_prefix_len FROM assignment_space_ipv6 WHERE id = ?")?;
        let mut rows = stmt.query(rusqlite::params![space_id])?;
        let row = rows.next()?;
        let space = match row {
            Some(row) => {
                let space = crate::ipv6::AssignmentSpaceIpv6 {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    space_visibility: row.get(3)?,
                    ipv6_prefix: row.get(4)?,
                    ipv6_prefix_len: row.get(5)?,
                };
                Some(space)
            },
            None => None,
        };
        let space = if let Some(space) = space {
            space
        } else {
            return Err(Error::new(ErrorKind::NotFound, "Space not found".to_string()));
        };
        Ok(space)
    }

    fn get_spaces(&self) -> Result<Vec<crate::ipv6::AssignmentSpaceIpv6>, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, space_visibility, ipv6_prefix, ipv6_prefix_len FROM assignment_space_ipv6 ORDER BY ipv6_prefix ASC")?;
        let mut rows = stmt.query(rusqlite::params![])?;
        let mut spaces = Vec::new();
        while let Some(row) = rows.next()? {
            let space = crate::ipv6::AssignmentSpaceIpv6 {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                space_visibility: row.get(3)?,
                ipv6_prefix: row.get(4)?,
                ipv6_prefix_len: row.get(5)?,
            };
            spaces.push(space);
        }
        Ok(spaces)
    }

    fn create_space(&self, space: &crate::ipv6::AssignmentSpaceIpv6) -> Result<i32, Error> {
        use crate::ipv6::ipv6_network_address;
        use crate::ipv6::ipv6_broadcast_address;

        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction()?;

        {
            let ipv6_network = ipv6_network_address(space.ipv6_prefix, (space.ipv6_prefix_len & 255) as u8);
            let ipv6_broadcast = ipv6_broadcast_address(space.ipv6_prefix, (space.ipv6_prefix_len & 255) as u8);
            let mut stmt = tx.prepare(
                "SELECT COUNT(*) FROM assignment_space_ipv6 
                WHERE ipv6_prefix >= ? AND ipv6_prefix <= ?"
            )?;
            let count: i32 = stmt.query_row(
                rusqlite::params![
                    ipv6_network, ipv6_broadcast,
                ],
                |row| row.get(0)
            )?;

            if count > 0 {
                return Err(Error::new(ErrorKind::InvalidInput, "Overlapping space exists".to_string()));
            }
        }

        for i in 0..space.ipv6_prefix_len {
            let ipv6_network = ipv6_network_address(space.ipv6_prefix, (i & 255) as u8);
            let mut stmt = tx.prepare(
                "SELECT COUNT(*) FROM assignment_space_ipv6 
                WHERE ipv6_prefix = ? AND ipv6_prefix_len = ?"
            )?;
            let count: i32 = stmt.query_row(
                rusqlite::params![
                    ipv6_network, i,
                ],
                |row| row.get(0)
            )?;

            if count > 0 {
                return Err(Error::new(ErrorKind::InvalidInput, "Overlapping space exists".to_string()));
            }
        }

        {
            let mut stmt = tx.prepare(
                "INSERT INTO assignment_space_ipv6 (name, description, space_visibility, ipv6_prefix, ipv6_prefix_len) 
                VALUES (?, ?, ?, ?, ?)"
            )?;
            stmt.execute(rusqlite::params![
                space.name, space.description, space.space_visibility, space.ipv6_prefix, space.ipv6_prefix_len
            ])?;
        }
        
        tx.commit()?;
        
        let id = conn.last_insert_rowid();
        Ok(id as i32)
    }

    fn update_space(&self, id: i32, name: &str, description: &str) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("UPDATE assignment_space_ipv6 SET name = ?, description = ? WHERE id = ?")?;
        stmt.execute(rusqlite::params![name, description, id])?;
        Ok(())
    }

    fn delete_space(&self, space_id: i32) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("DELETE FROM assignment_space_ipv6 WHERE id = ?")?;
        stmt.execute(rusqlite::params![space_id])?;
        Ok(())    
    }
    fn get_pool(&self, pool_id: i32) -> Result<crate::ipv6::AssignmentPoolIpv6, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, pool_visibility, ipv6_prefix, ipv6_prefix_len, assignment_space_id FROM assignment_pool_ipv6 WHERE id = ?")?;
        let mut rows = stmt.query(rusqlite::params![pool_id])?;
        let row = rows.next()?;
        let pool = match row {
            Some(row) => {
                let pool = crate::ipv6::AssignmentPoolIpv6 {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    pool_visibility: row.get(3)?,
                    ipv6_prefix: row.get(4)?,
                    ipv6_prefix_len: row.get(5)?,
                    assignment_space_id: row.get(6)?,
                };
                Some(pool)
            },
            None => None,
        };
        let pool = if let Some(pool) = pool {
            pool
        } else {
            return Err(Error::new(ErrorKind::NotFound, "Pool not found".to_string()));
        };
        Ok(pool)
    }

    fn get_pools(&self, space_id: i32) -> Result<Vec<crate::ipv6::AssignmentPoolIpv6>, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, pool_visibility, ipv6_prefix, ipv6_prefix_len, assignment_space_id FROM assignment_pool_ipv6 WHERE assignment_space_id = ? ORDER BY ipv6_prefix ASC")?;
        let mut rows = stmt.query(rusqlite::params![space_id])?;
        let mut pools = Vec::new();
        while let Some(row) = rows.next()? {
            let pool = crate::ipv6::AssignmentPoolIpv6 {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                pool_visibility: row.get(3)?,
                ipv6_prefix: row.get(4)?,
                ipv6_prefix_len: row.get(5)?,
                assignment_space_id: row.get(6)?,
            };
            pools.push(pool);
        }
        Ok(pools)
    }

    fn create_pool(&self, pool: &crate::ipv6::AssignmentPoolIpv6) -> Result<i32, Error> {
        use crate::ipv6::ipv6_network_address;
        use crate::ipv6::ipv6_broadcast_address;

        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction()?;
        {
            let pool_network = ipv6_network_address(pool.ipv6_prefix, (pool.ipv6_prefix_len & 255) as u8);
            let pool_broadcast = ipv6_broadcast_address(pool.ipv6_prefix, (pool.ipv6_prefix_len & 255) as u8);

            let mut stmt = tx.prepare(
                "SELECT ipv6_prefix, ipv6_prefix_len FROM assignment_space_ipv6 WHERE id = ?"
            )?;
            let mut rows = stmt.query(rusqlite::params![pool.assignment_space_id])?;
            let row = rows.next()?;
            let (space_network, space_broadcast) = match row {
                Some(row) => {
                    let space_prefix = row.get(0)?;
                    let space_prefix_len: u8 = row.get(1)?;
                    let space_network = ipv6_network_address(space_prefix, (space_prefix_len & 255) as u8);
                    let space_broadcast = ipv6_broadcast_address(space_prefix, (space_prefix_len & 255) as u8);
                    (space_network, space_broadcast)
                },
                None => return Err(Error::new(ErrorKind::NotFound, "Parent space not found".to_string())),
            };

            if pool_network < space_network || pool_broadcast > space_broadcast {
                return Err(Error::new(ErrorKind::InvalidInput, "Pool is not contained within the parent space".to_string()));
            }
        }

        {
            let ipv6_network = ipv6_network_address(pool.ipv6_prefix, (pool.ipv6_prefix_len & 255) as u8);
            let ipv6_broadcast = ipv6_broadcast_address(pool.ipv6_prefix, (pool.ipv6_prefix_len & 255) as u8);
            let mut stmt = tx.prepare(
                "SELECT COUNT(*) FROM assignment_pool_ipv6 
                WHERE ipv6_prefix >= ? AND ipv6_prefix <= ?"
            )?;
            let count: i32 = stmt.query_row(
                rusqlite::params![
                    ipv6_network, ipv6_broadcast,
                ],
                |row| row.get(0)
            )?;

            if count > 0 {
                return Err(Error::new(ErrorKind::InvalidInput, "Overlapping pool exists".to_string()));
            }
        }

        for i in 0..pool.ipv6_prefix_len {
            let ipv6_network = ipv6_network_address(pool.ipv6_prefix, (i & 255) as u8);
            let mut stmt = tx.prepare(
                "SELECT COUNT(*) FROM assignment_pool_ipv6 
                WHERE ipv6_prefix = ? AND ipv6_prefix_len = ?"
            )?;
            let count: i32 = stmt.query_row(
                rusqlite::params![
                    ipv6_network, i,
                ],
                |row| row.get(0)
            )?;

            if count > 0 {
                return Err(Error::new(ErrorKind::InvalidInput, "Overlapping pool exists".to_string()));
            }
        }

        {
            let mut stmt = tx.prepare(
                "INSERT INTO assignment_pool_ipv6 (name, description, pool_visibility, ipv6_prefix, ipv6_prefix_len, assignment_space_id) 
                VALUES (?, ?, ?, ?, ?, ?)"
            )?;
            stmt.execute(rusqlite::params![
                pool.name, pool.description, pool.pool_visibility, pool.ipv6_prefix, pool.ipv6_prefix_len, pool.assignment_space_id
            ])?;
        }
        
        tx.commit()?;
        
        let id = conn.last_insert_rowid();
        Ok(id as i32)
    }

    fn update_pool(&self, id: i32, name: &str, description: &str) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("UPDATE assignment_pool_ipv6 SET name = ?, description = ? WHERE id = ?")?;
        stmt.execute(rusqlite::params![name, description, id])?;
        Ok(())
    }

    fn delete_pool(&self, pool_id: i32) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("DELETE FROM assignment_pool_ipv6 WHERE id = ?")?;
        stmt.execute(rusqlite::params![pool_id])?;
        Ok(())
    }
    fn get_assignment(&self, assignment_id: i32) -> Result<crate::ipv6::AssignmentIpv6, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, ipv6_prefix, ipv6_prefix_len, assignment_pool_id, assignment_visibility FROM assignment_ipv6 WHERE id = ?")?;
        let mut rows = stmt.query(rusqlite::params![assignment_id])?;
        let row = rows.next()?;
        let assignment = match row {
            Some(row) => {
                let assignment = crate::ipv6::AssignmentIpv6 {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    ipv6_prefix: row.get(3)?,
                    ipv6_prefix_len: row.get(4)?,
                    assignment_pool_id: row.get(5)?,
                    assignment_visibility: row.get(6)?,
                };
                Some(assignment)
            },
            None => None,
        };
        let assignment = if let Some(assignment) = assignment {
            assignment
        } else {
            return Err(Error::new(ErrorKind::NotFound, "Assignment not found".to_string()));
        };
        Ok(assignment)
    }

    fn get_assignments(&self, pool_id: i32) -> Result<Vec<crate::ipv6::AssignmentIpv6>, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, description, ipv6_prefix, ipv6_prefix_len, assignment_pool_id, assignment_visibility FROM assignment_ipv6 WHERE assignment_pool_id = ? ORDER BY ipv6_prefix ASC")?;
        let mut rows = stmt.query(rusqlite::params![pool_id])?;
        let mut assignments = Vec::new();
        while let Some(row) = rows.next()? {
            let assignment = crate::ipv6::AssignmentIpv6 {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                ipv6_prefix: row.get(3)?,
                ipv6_prefix_len: row.get(4)?,
                assignment_pool_id: row.get(5)?,
                assignment_visibility: row.get(6)?,
            };
            assignments.push(assignment);
        }
        Ok(assignments)
    }

    fn create_assignment(&self, assignment: &crate::ipv6::AssignmentIpv6) -> Result<i32, Error> {
        use crate::ipv6::ipv6_network_address;
        use crate::ipv6::ipv6_broadcast_address;

        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction()?;
        {
            let assignment_network = ipv6_network_address(assignment.ipv6_prefix, (assignment.ipv6_prefix_len & 255) as u8);
            let assignment_broadcast = ipv6_broadcast_address(assignment.ipv6_prefix, (assignment.ipv6_prefix_len & 255) as u8);

            let mut stmt = tx.prepare(
            "SELECT ipv6_prefix, ipv6_prefix_len FROM assignment_pool_ipv6 WHERE id = ?"
            )?;
            let mut rows = stmt.query(rusqlite::params![assignment.assignment_pool_id])?;
            let row = rows.next()?;
            let (pool_network, pool_broadcast) = match row {
                Some(row) => {
                    let pool_prefix = row.get(0)?;
                    let pool_prefix_len: u8 = row.get(1)?;
                    let pool_network = ipv6_network_address(pool_prefix, (pool_prefix_len & 255) as u8);
                    let pool_broadcast = ipv6_broadcast_address(pool_prefix, (pool_prefix_len & 255) as u8);
                    (pool_network, pool_broadcast)
                },
                None => return Err(Error::new(ErrorKind::NotFound, "Parent pool not found".to_string())),
            };

            if assignment_network < pool_network || assignment_broadcast > pool_broadcast {
            return Err(Error::new(ErrorKind::InvalidInput, "Assignment is not contained within the parent pool".to_string()));
            }
        }

        {
            let ipv6_network = ipv6_network_address(assignment.ipv6_prefix, (assignment.ipv6_prefix_len & 255) as u8);
            let ipv6_broadcast = ipv6_broadcast_address(assignment.ipv6_prefix, (assignment.ipv6_prefix_len & 255) as u8);
            let mut stmt = tx.prepare(
                "SELECT COUNT(*) FROM assignment_ipv6 
                WHERE ipv6_prefix >= ? AND ipv6_prefix <= ?"
            )?;
            let count: i32 = stmt.query_row(
                rusqlite::params![
                    ipv6_network, ipv6_broadcast,
                ],
                |row| row.get(0)
            )?;

            if count > 0 {
                return Err(Error::new(ErrorKind::InvalidInput, "Overlapping assignment exists".to_string()));
            }
        }

        for i in 0..assignment.ipv6_prefix_len {
            let ipv6_network = ipv6_network_address(assignment.ipv6_prefix, (i & 255) as u8);
            let mut stmt = tx.prepare(
                "SELECT COUNT(*) FROM assignment_ipv6 
                WHERE ipv6_prefix = ? AND ipv6_prefix_len = ?"
            )?;
            let count: i32 = stmt.query_row(
                rusqlite::params![
                    ipv6_network, i,
                ],
                |row| row.get(0)
            )?;

            if count > 0 {
                return Err(Error::new(ErrorKind::InvalidInput, "Overlapping assignment exists".to_string()));
            }
        }

        {
            let mut stmt = tx.prepare(
                "INSERT INTO assignment_ipv6 (name, description, ipv6_prefix, ipv6_prefix_len, assignment_pool_id, assignment_visibility) 
                VALUES (?, ?, ?, ?, ?, ?)"
            )?;
            stmt.execute(rusqlite::params![
                assignment.name, assignment.description, assignment.ipv6_prefix, assignment.ipv6_prefix_len, assignment.assignment_pool_id, assignment.assignment_visibility
            ])?;
        }

        tx.commit()?;

        let id = conn.last_insert_rowid();
        Ok(id as i32)
    }

    fn update_assignment(&self, id: i32, name: &str, description: &str) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("UPDATE assignment_ipv6 SET name = ?, description = ? WHERE id = ?")?;
        stmt.execute(rusqlite::params![name, description, id])?;
        Ok(())
    }

    fn delete_assignment(&self, assignment_id: i32) -> Result<(), Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("DELETE FROM assignment_ipv6 WHERE id = ?")?;
        stmt.execute(rusqlite::params![assignment_id])?;
        Ok(())
    }
}

