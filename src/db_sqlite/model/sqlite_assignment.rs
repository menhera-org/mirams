use crate::{
    assignment::{Assignment, AssignmentRange, AssignmentStore, ResourceType},
    db_sqlite::SqliteConnection,
    types::ObjectVisibility,
    Error, ErrorKind,
};
use r2d2_sqlite::rusqlite::{self, params, OptionalExtension};

#[derive(Debug, Clone)]
pub struct SqliteAssignmentStore {
    db: SqliteConnection,
}
impl SqliteAssignmentStore {
    pub fn new(db: SqliteConnection) -> Self {
        Self { db }
    }
}

pub(crate) fn decode_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Assignment> {
    let json: String = row.get(5)?;
    let range = serde_json::from_str::<AssignmentRange>(&json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(Assignment {
        id: row.get(0)?,
        parent_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        visibility: row.get(4)?,
        range,
    })
}

pub(crate) fn insert_assignment(
    conn: &rusqlite::Connection,
    assignment: &Assignment,
) -> Result<i32, Error> {
    let (start, end) = assignment.range.bounds()?;
    conn.execute("INSERT INTO assignments (parent_id, name, description, visibility, resource_type, range_json, range_start, range_end) VALUES (?, ?, ?, ?, ?, ?, ?, ?)", params![assignment.parent_id, assignment.name, assignment.description, assignment.visibility, assignment.range.resource_type().as_str(), serde_json::to_string(&assignment.range)?, start, end])?;
    i32::try_from(conn.last_insert_rowid())
        .map_err(|_| Error::new(ErrorKind::InternalError, "Assignment ID exhausted"))
}

fn validate_move(
    conn: &rusqlite::Connection,
    assignment: &Assignment,
    parent_id: Option<i32>,
) -> Result<(), Error> {
    if parent_id == assignment.parent_id {
        return Ok(());
    }
    if let Some(parent_id) = parent_id {
        let parent = conn.query_row("SELECT id, parent_id, name, description, visibility, range_json FROM assignments WHERE id = ?", [parent_id], decode_row).optional()?.ok_or_else(|| Error::new(ErrorKind::NotFound, "Destination assignment not found"))?;
        let cycle: bool = conn.query_row("WITH RECURSIVE ancestors(id, parent_id) AS (SELECT id, parent_id FROM assignments WHERE id = ? UNION ALL SELECT a.id, a.parent_id FROM assignments a JOIN ancestors ON a.id = ancestors.parent_id) SELECT EXISTS(SELECT 1 FROM ancestors WHERE id = ?)", params![parent_id, assignment.id], |row| row.get(0))?;
        if cycle {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "An assignment cannot be moved into itself or its descendants",
            ));
        }
        if !parent.range.contains(&assignment.range)? {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Assignment must be within the destination range and use the same resource type",
            ));
        }
        let (start, end) = assignment.range.bounds()?;
        let overlap: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM assignments WHERE parent_id = ? AND id != ? AND range_start <= ? AND range_end >= ?)", params![parent_id, assignment.id, end, start], |row| row.get(0))?;
        if overlap {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Overlapping sibling assignment exists at destination",
            ));
        }
    }
    Ok(())
}

impl AssignmentStore for SqliteAssignmentStore {
    fn get_assignment(&self, id: i32) -> Result<Assignment, Error> {
        self.db.get_conn()?.query_row("SELECT id, parent_id, name, description, visibility, range_json FROM assignments WHERE id = ?", [id], decode_row).optional()?.ok_or_else(|| Error::new(ErrorKind::NotFound, "Assignment not found"))
    }

    fn get_assignments(
        &self,
        parent_id: Option<i32>,
        resource_type: Option<ResourceType>,
    ) -> Result<Vec<Assignment>, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, parent_id, name, description, visibility, range_json FROM assignments WHERE parent_id IS ? AND (? IS NULL OR resource_type = ?) ORDER BY resource_type, range_start, range_end, id")?;
        let resource_type = resource_type.map(ResourceType::as_str);
        let rows = stmt.query_map(params![parent_id, resource_type, resource_type], decode_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn get_ancestors(&self, id: i32) -> Result<Vec<Assignment>, Error> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare("WITH RECURSIVE ancestors(id, parent_id, depth) AS (SELECT id, parent_id, 0 FROM assignments WHERE id = ? UNION ALL SELECT a.id, a.parent_id, ancestors.depth + 1 FROM assignments a JOIN ancestors ON a.id = ancestors.parent_id) SELECT a.id, a.parent_id, a.name, a.description, a.visibility, a.range_json FROM ancestors JOIN assignments a ON a.id = ancestors.id WHERE ancestors.depth > 0 ORDER BY ancestors.depth DESC")?;
        let rows = stmt.query_map([id], decode_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn is_public(&self, id: i32) -> Result<bool, Error> {
        Ok(self.db.get_conn()?.query_row("WITH RECURSIVE ancestors(id, parent_id, visibility) AS (SELECT id, parent_id, visibility FROM assignments WHERE id = ? UNION ALL SELECT a.id, a.parent_id, a.visibility FROM assignments a JOIN ancestors ON a.id = ancestors.parent_id) SELECT COUNT(*) > 0 AND COALESCE(MAX(visibility), 1) = 0 FROM ancestors", [id], |row| row.get(0))?)
    }

    fn create_assignment(&self, assignment: &Assignment) -> Result<i32, Error> {
        let (start, end) = assignment.range.bounds()?;
        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        if let Some(parent_id) = assignment.parent_id {
            let parent = tx.query_row("SELECT id, parent_id, name, description, visibility, range_json FROM assignments WHERE id = ?", [parent_id], decode_row).optional()?.ok_or_else(|| Error::new(ErrorKind::NotFound, "Parent assignment not found"))?;
            if !parent.range.contains(&assignment.range)? {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Assignment must be within its parent range and use the same resource type",
                ));
            }
            let overlap: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM assignments WHERE parent_id = ? AND range_start <= ? AND range_end >= ?)", params![parent_id, end, start], |row| row.get(0))?;
            if overlap {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Overlapping sibling assignment exists",
                ));
            }
        }
        let id = insert_assignment(&tx, assignment)?;
        tx.commit()?;
        Ok(id)
    }

    fn update_assignment(
        &self,
        id: i32,
        name: &str,
        description: &str,
        visibility: Option<ObjectVisibility>,
    ) -> Result<(), Error> {
        let changed = self.db.get_conn()?.execute("UPDATE assignments SET name = ?, description = ?, visibility = COALESCE(?, visibility) WHERE id = ?", params![name, description, visibility, id])?;
        if changed == 0 {
            return Err(Error::new(ErrorKind::NotFound, "Assignment not found"));
        }
        Ok(())
    }

    fn move_assignment(&self, id: i32, parent_id: Option<i32>) -> Result<(), Error> {
        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let assignment = tx.query_row("SELECT id, parent_id, name, description, visibility, range_json FROM assignments WHERE id = ?", [id], decode_row).optional()?.ok_or_else(|| Error::new(ErrorKind::NotFound, "Assignment not found"))?;
        validate_move(&tx, &assignment, parent_id)?;
        tx.execute(
            "UPDATE assignments SET parent_id = ? WHERE id = ?",
            params![parent_id, id],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn get_move_targets(&self, id: i32) -> Result<Vec<Assignment>, Error> {
        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction()?;
        let assignment = tx.query_row("SELECT id, parent_id, name, description, visibility, range_json FROM assignments WHERE id = ?", [id], decode_row).optional()?.ok_or_else(|| Error::new(ErrorKind::NotFound, "Assignment not found"))?;
        let (start, end) = assignment.range.bounds()?;
        let candidates = {
            let mut stmt = tx.prepare("SELECT id, parent_id, name, description, visibility, range_json FROM assignments WHERE resource_type = ? AND range_start <= ? AND range_end >= ? AND id != ? ORDER BY range_start, range_end, name, id")?;
            let rows = stmt.query_map(
                params![assignment.range.resource_type().as_str(), start, end, id],
                decode_row,
            )?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };
        let mut targets = Vec::new();
        for candidate in candidates {
            match validate_move(&tx, &assignment, Some(candidate.id)) {
                Ok(()) => targets.push(candidate),
                Err(error) if matches!(error.kind(), ErrorKind::InvalidInput) => (),
                Err(error) => return Err(error),
            }
        }
        tx.commit()?;
        Ok(targets)
    }

    fn delete_assignment(&self, id: i32) -> Result<(), Error> {
        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        // A deferred FK and one recursive DELETE avoid SQLite's cascading-trigger
        // depth limit, so deep trees can be removed in one transaction.
        let changed = tx.execute("WITH RECURSIVE subtree(id) AS (SELECT id FROM assignments WHERE id = ? UNION ALL SELECT a.id FROM assignments a JOIN subtree ON a.parent_id = subtree.id) DELETE FROM assignments WHERE id IN (SELECT id FROM subtree)", [id])?;
        if changed == 0 {
            return Err(Error::new(ErrorKind::NotFound, "Assignment not found"));
        }
        tx.commit()?;
        Ok(())
    }
}
