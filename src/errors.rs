use std::sync::PoisonError;
use serde_json::error::Error as SerdeError;

error_chain! {
    errors {
        RecordNotFound(t: String) {
            description("Record not found")
            display("Record not found: '{}'", t)
        }
        Deadlock(t: String) {
            description("Database was poisoned")
            display("Database was poisoned: '{}'", t)
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Error {
        Error::from_kind(ErrorKind::Deadlock(format!("{}", err)))
    }
}
