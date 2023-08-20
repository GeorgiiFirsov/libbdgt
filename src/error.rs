use gpgme;


/// Structure, that describes all errors in bdgt.
#[derive(Debug, PartialEq)]
pub struct Error {
    msg: String,
    extra: String
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, r#"Message: "{}" (extra: "{}")"#, self.msg, self.extra)
    }
}


impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.msg
    }
}


impl From<gpgme::Error> for Error {
    fn from(value: gpgme::Error) -> Self {
        let msg = value
            .description()
            .to_string();

        let extra = format!("code: {}", value.code());

        Error { 
            msg: msg, 
            extra: extra 
        }
    }
}


/// Crate-specific alias for [`std::result::Result`] instantiated 
/// with [`crate::error::Error`].
pub type Result<T> = std::result::Result<T, Error>;
