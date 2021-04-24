use std::error;
use std::fmt;
use std::io;
use colored::*;

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
    AccessibilityError {
        path: String,
        offender: String,
        description: String,
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
            CheckError::ContentError {
                path,
                offender,
                description,
            } => {
                write!(
                    f,
                    "{}: [{}{}], in file {}",
                    &"Found content error".red(),&description, &offender, &path
                )
            }
            CheckError::AccessibilityError {
                path,
                offender,
                description,
            } => {
                write!(
                    f,
                    "{}: [{}{}], in file {}",
                    &"Found accessiblity error".red(),&description, &offender, &path
                )
            }
            CheckError::ForbiddenFile { path } => write!(f, "{}: {}", &"Found forbidden file".red(),path),
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
            (
                AccessibilityError {
                    path: ps,
                    offender: offs,
                    description: dess,
                },
                AccessibilityError {
                    path: po,
                    offender: offo,
                    description: deso,
                },
            ) => ps == po && offs == offo && dess == deso,
            (ForbiddenFile { path: ps }, ForbiddenFile { path: po }) => ps == po,
            (Io(_), Io(_)) => true, //all io erros are equal for our purposes
            _ => false,
        }
    }
}
