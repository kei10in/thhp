error_chain! {
    errors {
        Incomplete
        InvalidHeaderFormat
    }
}

pub use ErrorKind::*;

impl PartialEq<ErrorKind> for ErrorKind {
    fn eq(&self, other: &ErrorKind) -> bool {
        match (self, other) {
            (&Incomplete, &Incomplete) => true,
            (&InvalidHeaderFormat, &InvalidHeaderFormat) => true,
            _ => false,
        }
    }
}
