error_chain! {
    errors {
        InvalidFormat
    }
}

pub use ErrorKind::*;

impl PartialEq<ErrorKind> for ErrorKind {
    fn eq(&self, other: &ErrorKind) -> bool {
        match (self, other) {
            (&InvalidFormat, &InvalidFormat) => true,
            _ => false,
        }
    }
}
