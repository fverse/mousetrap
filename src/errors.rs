use std::{error::Error, fmt};

/// Represents errors that may occur while attempting to establish a connection.
///
/// TODO: Should provide more specific information about what went wrong during the
/// connection process.
#[derive(Debug)]
pub enum ConnectionError {
    InvalidSocketPath,
    ConnectionRefused,
    ConnectionNotEstablished,
    FurtherAuthenticationRequired,
    InvalidResponseFromServer,
}

impl Error for ConnectionError {}

impl From<ParseError> for ConnectionError {
    fn from(_: ParseError) -> Self {
        ConnectionError::ConnectionNotEstablished
    }
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSocketPath => {
                write!(f, "Failed to connect: The socket path is invalid")
            }
            Self::ConnectionNotEstablished => {
                write!(
                    f,
                    "Failed to connect: Connection to the server is not established"
                )
            }
            Self::ConnectionRefused => {
                write!(f, "Failed to connect: Connection is refused by the server")
            }
            Self::FurtherAuthenticationRequired => {
                write!(f, "Failed to connect: Further authentication is required")
            }
            Self::InvalidResponseFromServer => {
                write!(
                    f,
                    "Failed to connect: Invalid response from server on connection setup"
                )
            }
        }
    }
}

/// Represents errors that may occur while serializing or deserializing sequence of x11 bytes.
#[derive(Debug)]
pub enum ParseError {
    OutOfBound,
    NotEnoughData,
    OverFlow,
    Failed,
    InvalidLength,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OutOfBound => {
                write!(
                    formatter,
                    "Failed to parse: The provided range is out of bound"
                )
            }
            Self::NotEnoughData => {
                formatter.write_str("Failed to parse: Not enough data to parse: Out of bound")
            }
            Self::OverFlow => {
                write!(formatter, "Failed to parse: data length overflow")
            }
            Self::Failed => {
                write!(formatter, "Failed to parse")
            }
            Self::InvalidLength => {
                write!(formatter, "Failed to parse: Invalid data length")
            }
        }
    }
}
