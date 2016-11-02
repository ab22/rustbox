use std::net;
use std::io;
use std::error::Error;
use std::fmt::{self, Display};


// UnhexlifyError describes an error when trying to convert a string to a Vec<u8>.
#[derive(Debug, PartialEq)]
pub enum UnhexlifyError {
    OddCharacterCount(usize),
    InvalidHexDigit(char),
}

impl Error for UnhexlifyError {
    fn description(&self) -> &str {
        match *self {
            UnhexlifyError::OddCharacterCount(_) => "invalid string length",
            UnhexlifyError::InvalidHexDigit(_) => "could not convert character to hex digit",
        }
    }

    fn cause(&self) -> Option<&Error> {
        Some(self as &Error)
    }
}

impl Display for UnhexlifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UnhexlifyError::InvalidHexDigit(c) => {
                write!(f, "invalid character '{}': {}", c, self.description())
            }
            UnhexlifyError::OddCharacterCount(x) => {
                write!(f, "string length '{}': {}", x, self.description())
            }
        }
    }
}


// CreateClientError is used to describe the different types of errors when trying to
// create a Mikrotik Client.
//
// AddrParseError happens when trying to parse a string to IPv4.
//
// IoError is returned when there was an error creating the inner stream socket used
// for communication with the router.
#[derive(Debug)]
pub enum CreateClientError {
    AddrParseError(net::AddrParseError),
    IoError(io::Error),
}

impl Error for CreateClientError {
    fn description(&self) -> &str {
        match *self {
            CreateClientError::AddrParseError(ref e) => e.description(),
            CreateClientError::IoError(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        Some(match *self {
            CreateClientError::AddrParseError(ref e) => e,
            CreateClientError::IoError(ref e) => e,
        })
    }
}

impl Display for CreateClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CreateClientError::AddrParseError(ref e) => e.fmt(f),
            CreateClientError::IoError(ref e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for CreateClientError {
    fn from(err: io::Error) -> CreateClientError {
        CreateClientError::IoError(err)
    }
}

impl From<net::AddrParseError> for CreateClientError {
    fn from(err: net::AddrParseError) -> CreateClientError {
        CreateClientError::AddrParseError(err)
    }
}


// MikroTik Error.
#[derive(Debug)]
pub enum MikrotikError {
    UnhexlifyError(UnhexlifyError),
    IoError(io::Error),
    Fatal(String),
    Trap { category: u8, msg: String },
}

impl Error for MikrotikError {
    fn description(&self) -> &str {
        match *self {
            MikrotikError::UnhexlifyError(ref e) => e.description(),
            MikrotikError::IoError(ref e) => e.description(),
            MikrotikError::Fatal(_) => "a fatal error has ocurred",
            MikrotikError::Trap { category, .. } => {
                match category {
                    0 => "missing item or command",
                    1 => "argument value failure",
                    2 => "execution of command interrupted",
                    3 => "scripting related failure",
                    4 => "general failure",
                    5 => "API related failure",
                    6 => "TTY related failure",
                    7 => "value generated with :return command",
                    _ => "",
                }
            }
        }
    }

    fn cause(&self) -> Option<&Error> {
        Some(match *self {
            MikrotikError::UnhexlifyError(ref e) => e,
            MikrotikError::IoError(ref e) => e,
            _ => self as &Error,
        })
    }
}

impl Display for MikrotikError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MikrotikError::UnhexlifyError(ref e) => e.fmt(f),
            MikrotikError::IoError(ref e) => e.fmt(f),
            MikrotikError::Fatal(ref msg) => write!(f, "{}: {}", self.description(), msg),
            MikrotikError::Trap { category, ref msg } => {
                write!(f, "[trap_id:{}] {} - {}", category, self.description(), msg)
            }
        }
    }
}

impl From<io::Error> for MikrotikError {
    fn from(err: io::Error) -> MikrotikError {
        MikrotikError::IoError(err)
    }
}

impl From<UnhexlifyError> for MikrotikError {
    fn from(err: UnhexlifyError) -> MikrotikError {
        MikrotikError::UnhexlifyError(err)
    }
}
