use std::fmt;

#[derive(Debug)]
pub enum BwError {
    NoInstallMethod { app: String },
    CommandFailed { cmd: String, status: Option<i32> },
    Io(std::io::Error),
    NotRoot,
    Other(String),
}

impl fmt::Display for BwError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BwError::NoInstallMethod { app } => {
                write!(
                    f,
                    "no viable install method found for '{app}' on this system"
                )
            }
            BwError::CommandFailed { cmd, status } => {
                write!(f, "command failed ({cmd}), exit status: {status:?}")
            }
            BwError::Io(e) => write!(f, "io error: {e}"),
            BwError::NotRoot => write!(
                f,
                "this action requires root privileges (try running with sudo)"
            ),
            BwError::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for BwError {}

impl From<std::io::Error> for BwError {
    fn from(e: std::io::Error) -> Self {
        BwError::Io(e)
    }
}

pub type BwResult<T> = Result<T, BwError>;
