
use serde::{
    Serialize,
    Deserialize,
};

use std::fmt::{
    Formatter,
    Display,
};

use std::any::Any;

use std::error::Error as StdError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[repr(i32)]
pub enum ObjectVisibility {
    /// Child assignments visible to everyone
    Public = 0,

    /// Child assignments only visible to logged-in users
    Private = 1,
}

impl TryFrom<i32> for ObjectVisibility {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ObjectVisibility::Public),
            1 => Ok(ObjectVisibility::Private),
            _ => Err(format!("Invalid visibility value: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ErrorKind {
    DatabaseError,
    NotFound,
    InvalidInput,
    InternalError,
    Other,
}

pub(crate) trait ErrorWithKind {
    fn kind(&self) -> ErrorKind;
}

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl ToString) -> Self {
        Error {
            kind,
            message: message.to_string(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl<T> From<T> for Error
where
    T: StdError + Send + Sync + Any + 'static,
{
    fn from(err: T) -> Self {
        let message = err.to_string();
        let kind = if let Some(err) = (&err as &dyn Any).downcast_ref::<&(dyn ErrorWithKind + 'static)>() {
            err.kind()
        } else {
            ErrorKind::Other
        };
        Error {
            kind,
            message,
        }
    }
}
