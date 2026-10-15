use crate::{assignment::AssignmentStore, user::UserStore};

pub trait DbConnection {
    fn user_store(&self) -> Box<dyn UserStore>;
    fn assignment_store(&self) -> Box<dyn AssignmentStore>;
}

#[derive(Debug, Clone)]
pub struct Store<T>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    db: T,
}

impl<T> Store<T>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    pub fn new(db: T) -> Self {
        Self { db }
    }
    pub fn users(&self) -> Box<dyn UserStore> {
        self.db.user_store()
    }
    pub fn assignments(&self) -> Box<dyn AssignmentStore> {
        self.db.assignment_store()
    }
}
