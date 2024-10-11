
use crate::user::UserStore;
use crate::ipv4::Ipv4AssignmentStore;
use crate::ipv6::Ipv6AssignmentStore;
use crate::asn::AsnAssignmentStore;

pub trait DbConnection {
    fn user_store(&self) -> Box<dyn UserStore>;

    fn ipv4_assignment_store(&self) -> Box<dyn Ipv4AssignmentStore>;

    fn ipv6_assignment_store(&self) -> Box<dyn Ipv6AssignmentStore>;

    fn asn_assignment_store(&self) -> Box<dyn AsnAssignmentStore>;
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
        Store { db }
    }

    pub fn users(&self) -> Box<dyn UserStore> {
        self.db.user_store()
    }

    pub fn ipv4_assignments(&self) -> Box<dyn Ipv4AssignmentStore> {
        self.db.ipv4_assignment_store()
    }

    pub fn ipv6_assignments(&self) -> Box<dyn Ipv6AssignmentStore> {
        self.db.ipv6_assignment_store()
    }

    pub fn asn_assignments(&self) -> Box<dyn AsnAssignmentStore> {
        self.db.asn_assignment_store()
    }
}
