use super::*;
use crate::assignment::{Assignment, AssignmentRange};
use std::collections::HashMap;

pub(super) fn migrate_assignments(tx: &rusqlite::Transaction<'_>) -> Result<(), Error> {
    tx.execute_batch(r#"
        CREATE TABLE assignments (
            id INTEGER PRIMARY KEY,
            parent_id INTEGER REFERENCES assignments(id) DEFERRABLE INITIALLY DEFERRED,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            visibility INTEGER NOT NULL CHECK(visibility IN (0, 1)),
            resource_type TEXT NOT NULL CHECK(resource_type IN ('asn', 'ipv4', 'ipv6')),
            range_json TEXT NOT NULL,
            range_start BLOB NOT NULL CHECK(length(range_start) = 16),
            range_end BLOB NOT NULL CHECK(length(range_end) = 16),
            CHECK(range_start <= range_end)
        );
        CREATE INDEX assignments_parent ON assignments(parent_id, resource_type, range_start, range_end);
    "#)?;
    for family in ["asn", "ipv4", "ipv6"] {
        let mut mapped: HashMap<(usize, i32), Assignment> = HashMap::new();
        for (level, kind) in ["space", "pool", "assignment"].into_iter().enumerate() {
            let table = if level == 2 {
                format!("assignment_{family}")
            } else {
                format!("assignment_{kind}_{family}")
            };
            let parent_column = match level {
                0 => "NULL",
                1 => "assignment_space_id",
                _ => "assignment_pool_id",
            };
            let visibility_column = format!("{kind}_visibility");
            let range_columns = if family == "asn" {
                if level == 2 {
                    "asn".to_string()
                } else {
                    "asn_from, asn_to".to_string()
                }
            } else {
                format!("{family}_prefix, {family}_prefix_len")
            };
            let records = {
                let mut stmt = tx.prepare(&format!("SELECT id, {parent_column}, name, description, {visibility_column}, {range_columns} FROM {table} ORDER BY id"))?;
                let rows = stmt.query_map([], |row| {
                    let range = match family {
                        "asn" => {
                            let first: u32 = row.get(5)?;
                            AssignmentRange::Asn {
                                asn_from: first,
                                asn_to: if level == 2 { first } else { row.get(6)? },
                            }
                        }
                        "ipv4" => {
                            let prefix = row.get(5)?;
                            let len: i32 = row.get(6)?;
                            AssignmentRange::Ipv4 {
                                ipv4_prefix: crate::ipv4::ipv4_network_address(prefix, len as u8),
                                ipv4_prefix_len: len,
                            }
                        }
                        _ => {
                            let prefix = row.get(5)?;
                            let len: i32 = row.get(6)?;
                            AssignmentRange::Ipv6 {
                                ipv6_prefix: crate::ipv6::ipv6_network_address(prefix, len as u8),
                                ipv6_prefix_len: len,
                            }
                        }
                    };
                    Ok((
                        row.get::<_, i32>(0)?,
                        row.get::<_, Option<i32>>(1)?,
                        Assignment {
                            id: 0,
                            parent_id: None,
                            name: row.get(2)?,
                            description: row.get(3)?,
                            visibility: row.get(4)?,
                            range,
                        },
                    ))
                })?;
                rows.collect::<rusqlite::Result<Vec<_>>>()?
            };
            for (old_id, old_parent, mut assignment) in records {
                if let Some(parent) = old_parent {
                    let parent = mapped.get(&(level - 1, parent)).ok_or_else(|| {
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!("Cannot migrate {family} {kind} {old_id}: parent not found"),
                        )
                    })?;
                    if !parent.range.contains(&assignment.range)? {
                        return Err(Error::new(ErrorKind::InvalidInput, format!("Cannot migrate {family} {kind} {old_id}: range is outside its parent")));
                    }
                    assignment.parent_id = Some(parent.id);
                }
                // Preserve existing allocations even if historical bugs allowed
                // siblings to overlap. New allocations still enforce overlap rules.
                assignment.id = model::sqlite_assignment::insert_assignment(tx, &assignment)?;
                mapped.insert((level, old_id), assignment);
            }
        }
        for table in [
            format!("assignment_{family}"),
            format!("assignment_pool_{family}"),
            format!("assignment_space_{family}"),
        ] {
            tx.execute_batch(&format!("DROP TABLE {table}"))?;
        }
    }
    Ok(())
}
