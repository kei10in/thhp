error_chain! {
    errors {
        InvalidFieldName
        InvalidFieldValue
        InvalidNewLine
        InvalidVersion
        InvalidMethod
        InvalidPath
        InvalidStatusCode
        InvalidReasonPhrase
        OutOfCapacity
    }
}

pub use ErrorKind::*;

impl PartialEq<ErrorKind> for ErrorKind {
    fn eq(&self, other: &ErrorKind) -> bool {
        match (self, other) {
            (&InvalidFieldName, &InvalidFieldName) => true,
            (&InvalidFieldValue, &InvalidFieldValue) => true,
            (&InvalidNewLine, &InvalidNewLine) => true,
            (&InvalidVersion, &InvalidVersion) => true,
            (&InvalidMethod, &InvalidMethod) => true,
            (&InvalidPath, &InvalidPath) => true,
            (&InvalidStatusCode, &InvalidStatusCode) => true,
            (&InvalidReasonPhrase, &InvalidReasonPhrase) => true,
            _ => false,
        }
    }
}
