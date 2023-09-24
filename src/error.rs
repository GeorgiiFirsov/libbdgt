/// Structure, that describes all errors in libbdgt.
#[derive(Debug, PartialEq)]
pub struct Error {
    msg: String,
    extra: String
}


/// Crate-specific alias for [`std::result::Result`] instantiated 
/// with [`crate::error::Error`].
pub type Result<T> = std::result::Result<T, Error>;


impl Error {
    /// Constructs an error from message.
    /// 
    /// * `msg` - error message as something convertible into a [`alloc::string::String`]
    pub fn from_message<M>(msg: M) -> Self 
    where
        M: Into<String>
    {
        Error { 
            msg: msg.into(), 
            extra: String::new() 
        }
    }

    /// Constructs an error from message with some extra information.
    /// 
    /// * `msg` - error message as something convertible into a [`alloc::string::String`]
    /// * `extra` - extra information as something convertible into a [`alloc::string::String`]
    pub fn from_message_with_extra<M, E>(msg: M, extra: E) -> Self
    where
        M: Into<String>,
        E: Into<String>
    {
        Error { 
            msg: msg.into(), 
            extra: extra.into() 
        }
    }
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
        let msg = value.to_string();
        let extra = format!("code: {}", value.code());

        Error { 
            msg: msg, 
            extra: extra 
        }
    }
}


impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        let msg = value.to_string();

        Error {
            msg: msg,
            extra: String::new()
        }
    }
}


impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        let msg = value.to_string();

        Error {
            msg: msg,
            extra: String::new()
        }
    }
}
