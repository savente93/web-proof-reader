extern crate clap;
extern crate scraper;
extern crate selectors;
extern crate walkdir;
#[macro_use]
extern crate lazy_static;
extern crate regex;

mod html;

use std::error;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

static TEST_SITE: &str = "public/";

pub enum FileType<'a> {
    Html(&'a Path),
    Ignored,
}

#[derive(Debug)]
pub enum CheckError {
    ContentError,
    BrokenLink,
    EmptyBody,
    InvalidHtml,
    Io(io::Error),
}

impl From<io::Error> for CheckError {
    fn from(err: io::Error) -> CheckError {
        CheckError::Io(err)
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CheckError")
    }
}

impl error::Error for CheckError {}

pub fn check_file(file: FileType) -> Result<(), CheckError> {
    match file {
        FileType::Ignored => Ok(()),
        FileType::Html(p) => html::check_html_file(p),
    }
}

pub fn typify(path: Option<&Path>) -> FileType {
    match path {
        None => FileType::Ignored,
        Some(p) => {
            if p.extension().unwrap_or(&OsString::from("")) == "html" {
                FileType::Html(p)
            } else {
                FileType::Ignored
            }
        }
    }
}

fn main() -> Result<(), CheckError> {
    //TODO: make this loop parallel/async
    for entry in WalkDir::new(&TEST_SITE).into_iter().filter_map(|e| e.ok()) {
        check_file(typify(Some(entry.path())))?;
    }

    Ok(())
}
