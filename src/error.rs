use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum CheckError {
    ForbiddenFile {
        path: String,
    },
    ContentError {
        path: String,
        offender: String,
        description: String,
    },
    BrokenLink {
        path: String,
        link: String,
    },
    Io(io::Error),
}

impl From<io::Error> for CheckError {
    fn from(err: io::Error) -> CheckError {
        CheckError::Io(err)
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CheckError::BrokenLink { path, link } => {
                write!(f, "Found broken link: {}, in file {}", &link, &path)
            }
            CheckError::ContentError {
                path,
                offender,
                description,
            } => {
                write!(
                    f,
                    "Found content error: [{}{}], in file {}",
                    &description, &offender, &path
                )
            }
            CheckError::ForbiddenFile { path } => write!(f, "Found forbidden file: {}", path),
            CheckError::Io(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for CheckError {}

// Mostly just for testing
impl PartialEq for CheckError {
    fn eq(&self, other: &Self) -> bool {
        use CheckError::*;
        match (self, other) {
            (
                ContentError {
                    path: ps,
                    offender: offs,
                    description: dess,
                },
                ContentError {
                    path: po,
                    offender: offo,
                    description: deso,
                },
            ) => ps == po && offs == offo && dess == deso,
            (BrokenLink { path: ps, link: ls }, BrokenLink { path: po, link: lo }) => {
                po == ps && ls == lo
            }
            (ForbiddenFile { path: ps }, ForbiddenFile { path: po }) => ps == po,
            (Io(_), Io(_)) => true, //all io erros are equal for our purposes
            _ => false,
        }
    }
}
