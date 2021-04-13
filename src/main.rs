#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate regex;
extern crate scraper;
extern crate selectors;
extern crate walkdir;

mod error;
mod html;

use std::ffi::OsString;
use std::path::Path;
use walkdir::WalkDir;

use crate::error::CheckError;
static TEST_SITE: &str = "public/";

pub enum FileType<'a> {
    Html(&'a Path),
    Ignored,
}

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

fn main() -> Result<(), String> {
    //TODO: make this loop parallel
    let (_, errors): (Vec<_>, Vec<_>) = WalkDir::new(&TEST_SITE)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| check_file(typify(Some(e.path()))))
        .partition(Result::is_ok);
    for err in &errors {
        match err {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err("Errors were found".to_string())
    }
}
