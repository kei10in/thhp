error_chain! {
    errors {
        Incomplete
        InvalidFormat
    }
}

pub use ErrorKind::*;

impl PartialEq<ErrorKind> for ErrorKind {
    fn eq(&self, other: &ErrorKind) -> bool {
        match (self, other) {
            (&Incomplete, &Incomplete) => true,
            (&InvalidFormat, &InvalidFormat) => true,
            _ => false,
        }
    }
}
