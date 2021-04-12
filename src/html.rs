use crate::scraper::{ElementRef, Html, Selector};
use crate::selectors::attr::CaseSensitivity;
use std::fs::read_to_string;
use std::path::Path;

use crate::CheckError;

use regex::Regex;
use std::collections::HashSet;
type CheckResult = Result<(), CheckError>;

pub fn check_html_file(path: &Path) -> CheckResult {
    check_for_forbidden_files(path)?;
    
    let contents = read_to_string(path)?;
    let html = Html::parse_document(&contents);

    check_forbidden_tags(path, &html)?;
    check_for_invalid_publish_dates(path, &html)?;
    Ok(())
}

fn extract_tag_name_from_url(url: &str) -> Option<String> {
    lazy_static! {
        static ref RE_TAGS: Regex =
            Regex::new(&"tags/([-a-zA-Z0-9]+)").unwrap();
    }
    RE_TAGS
        .captures_iter(&url)
        .next()?
        .get(1)
        .map_or(None, |m| Some(m.as_str().to_string()))
}

fn extract_iso_date(text: &str) -> Option<String> {
    lazy_static! {
        static ref RE_DATE: Regex = Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap();
    }
    RE_DATE
        .captures_iter(&text)
        .next()?
        .get(1)
        .map_or(None, |m| Some(m.as_str().to_string()))
}

fn check_for_forbidden_files(path: &Path ) -> CheckResult{
    //TODO impl making sure that files like base/tags/wip or base/blog/unpublished/whatever don't exist
    let forbidden_folders = {
        let mut set = HashSet::new();
        set.insert("unpublished");
        set.insert("publish-queue");
        set
    };
        
    for comp in path.components(){
        if forbidden_folders.contains(&comp.as_os_str().to_str().unwrap()){
            // println!("Found forbidden file: {}", path.display());
            return Err(CheckError::ForbiddenFile(path.display().to_string()));
        } 
    }
    
    Ok(())
}


fn check_for_invalid_publish_dates(path: &Path, document: &Html) -> CheckResult {
    let div_selector = Selector::parse("div.date").unwrap();
    for div in document.select(&div_selector) {
        if let Some(publish_date) = extract_iso_date(&div.text().collect::<Vec<_>>().join("")) {
            if publish_date == "0000-01-01" {
                // println!("Found forbidden publish date: {:#?}", &publish_date);
                return Err(CheckError::ContentError("Forbidden publish date".to_string(),path.display().to_string()));
            }
        } else {
            // println!("File has no publish date!");
                return Err(CheckError::ContentError("No publish date".to_string(),path.display().to_string()));
        }
    }
    Ok(())
}

fn check_forbidden_tags(path: &Path, document: &Html) -> CheckResult {
    let forbidden_tags = {
        let mut set: HashSet<String> = HashSet::new();
        set.insert("wip".to_string());
        set
    };

    // check for forbidden tags
    let div_selector = Selector::parse("div").unwrap();
    for div in document.select(&div_selector).filter(|elt: &ElementRef| {
        elt.value()
            .has_class("tags", CaseSensitivity::AsciiCaseInsensitive)
    }) {
        for elt in div
            .children()
            .filter_map(|child| match child.value().is_element() {
                true => Some(child.value().as_element().unwrap()),
                false => None,
            })
        {
            if let Some(url) = elt.attr("href") {
                let tag_name = extract_tag_name_from_url(&url).unwrap_or("".to_string());

                if forbidden_tags.contains(&tag_name) {
                    // println!("Found forbidden tag: {:#?}", &tag_name);
                    //return Err(CheckError::ForbiddenTag);

                return Err(CheckError::ContentError(format!("Link to forbidden tag {}", &tag_name),path.display().to_string()));
                }
            }
        }
    }

    Ok(())
}
