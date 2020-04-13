use std::error;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Deserialize(toml::de::Error),
    I3Establish(i3ipc::EstablishError),
    I3Message(i3ipc::MessageError),
    Io(std::io::Error),
    Launchpad(launchpad::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Deserialize(err) => write!(f, "deserialization error: {}", err),
            Error::I3Establish(err) => write!(f, "i3 establish error: {}", err),
            Error::I3Message(err) => write!(f, "i3 message error: {}", err),
            Error::Io(err) => write!(f, "io error: {}", err),
            Error::Launchpad(err) => write!(f, "launchpad error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Deserialize(err) => Some(err),
            Error::I3Establish(err) => Some(err),
            Error::I3Message(err) => Some(err),
            Error::Io(err) => Some(err),
            Error::Launchpad(err) => Some(err),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Deserialize(err)
    }
}

impl From<i3ipc::EstablishError> for Error {
    fn from(err: i3ipc::EstablishError) -> Self {
        Error::I3Establish(err)
    }
}

impl From<i3ipc::MessageError> for Error {
    fn from(err: i3ipc::MessageError) -> Self {
        Error::I3Message(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<launchpad::Error> for Error {
    fn from(err: launchpad::Error) -> Self {
        Error::Launchpad(err)
    }
}
