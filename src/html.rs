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
        static ref RE_TAGS: Regex = Regex::new(&"tags/([-a-zA-Z0-9]+)").unwrap();
    }
    RE_TAGS
        .captures_iter(&url)
        .next()?
        .get(1)
        .map(|m| m.as_str().to_string())
}

fn extract_iso_date(text: &str) -> Option<String> {
    lazy_static! {
        static ref RE_DATE: Regex = Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap();
    }
    RE_DATE
        .captures_iter(&text)
        .next()?
        .get(1)
        .map(|m| m.as_str().to_string())
}

fn check_for_forbidden_files(path: &Path) -> CheckResult {
    let forbidden_folders = {
        let mut set = HashSet::new();
        set.insert("unpublished");
        set.insert("publish-queue");
        set
    };

    for comp in path.components() {
        if forbidden_folders.contains(&comp.as_os_str().to_str().unwrap()) {
            return Err(CheckError::ForbiddenFile {
                path: path.display().to_string(),
            });
        }
    }

    Ok(())
}

fn check_for_invalid_publish_dates(path: &Path, document: &Html) -> CheckResult {
    let div_selector = Selector::parse("div.date").unwrap();
    let mut found_tag = false;

    for div in document.select(&div_selector) {
        found_tag = true;
        let div_text = &div.text().collect::<Vec<_>>().join("");
        println!("{}",&div_text);
        if let Some(publish_date) = extract_iso_date(div_text) {
            if publish_date == "0000-01-01" {
                return Err(CheckError::ContentError {
                    path: path.display().to_string(),
                    offender: publish_date,
                    description: "Forbidden publish date".to_string(),
                });
            }
        } else {
            return Err(CheckError::ContentError {
                path: path.display().to_string(),
                offender: "".to_string(),
                description: "Missing publish date".to_string(),
            });
        }
    }

    if !found_tag {
        return Err(CheckError::ContentError {
            path: path.display().to_string(),
            offender: "".to_string(),
            description: "Missing publish date".to_string(),
        });
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
                let tag_name = extract_tag_name_from_url(&url).unwrap_or_else(|| "".to_string());

                if forbidden_tags.contains(&tag_name) {
                    return Err(CheckError::ContentError {
                        path: path.display().to_string(),
                        offender: tag_name,
                        description: "Forbidden tag".to_string(),
                    });
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir, File};
    use tempfile::TempDir;
    use std::io::prelude::*;

    fn setup_test_wip_page() -> Html {
        let wip_page_contents = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8">
            <title>title</title>
        </head>
        <body>
            <main>
                <article>
                <header>
                    <h2>A work in progress</h2>
                    <div class="date">Published: 0000-01-01</div>
                    <hr>
                </header>
                <section>
                </section>
                <nav>
                </nav>
                <div class="tags"><a href="/tags/wip">#WIP</a>, 
                </article>
            </main>
        </body>
        </html>
        "#;

        Html::parse_document(wip_page_contents)
    }

    fn setup_test_page_without_pub_date() -> Html {
        let wip_page_contents = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8">
            <title>title</title>
        </head>
        <body>
            <main>
                <article>
                <header>
                    <h2>A work in progress</h2>
                    <div class="date">Published:</div>
                    <hr>
                </header>
                <section>
                </section>
                <nav>
                </nav>
                <div class="tags"><a href="/tags/wip">#WIP</a>, 
                </article>
            </main>
        </body>
        </html>
        "#;

        Html::parse_document(wip_page_contents)
    }

    fn setup_test_page_without_pub_date_tag() -> Html {
        let wip_page_contents = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8">
            <title>title</title>
        </head>
        <body>
            <main>
                <article>
                <header>
                    <h2>A work in progress</h2>
                    <hr>
                </header>
                <section>
                </section>
                <nav>
                </nav>
                <div class="tags"><a href="/tags/wip">#WIP</a>, 
                </article>
            </main>
        </body>
        </html>
        "#;

        Html::parse_document(wip_page_contents)
    }

    fn setup_test_correct_page() -> Html {
        let wip_page_contents = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8">
            <title>title</title>
        </head>
        <body>
            <main>
                <article>
                <header>
                    <h2>A "complete" web page</h2>
                    <div class="date">Published: 2021-04-13</div>
                    <hr>
                </header>
                <section>
                <p> if you enjoy this content, hit the subscribe button!</p>
                </section>
                <nav>
                </nav>
                <div class="tags"><a href="/tags/testing">#testing</a>, 
                </article>
            </main>
        </body>
        </html>
        "#;

        Html::parse_document(wip_page_contents)
    }

    #[test]
    fn test_discovers_forbidden_pub_date() -> Result<(), String> {
        let test_doc = setup_test_wip_page();
        let test_path = Path::new("wip.html");

        let res = check_for_invalid_publish_dates(&test_path, &test_doc);

        let expected_err = Err(CheckError::ContentError {
            path: "wip.html".to_string(),
            offender: "0000-01-01".to_string(),
            description: "Forbidden publish date".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_missing_pub_date() -> Result<(), String> {
        let test_doc = setup_test_page_without_pub_date();
        let test_path = Path::new("wip.html");

        let res = check_for_invalid_publish_dates(&test_path, &test_doc);

        let expected_err = Err(CheckError::ContentError {
            path: "wip.html".to_string(),
            offender: "".to_string(),
            description: "Missing publish date".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_missing_pub_date_tag() -> Result<(), String> {
        let test_doc = setup_test_page_without_pub_date_tag();
        let test_path = Path::new("wip.html");

        let res = check_for_invalid_publish_dates(&test_path, &test_doc);

        let expected_err = Err(CheckError::ContentError {
            path: "wip.html".to_string(),
            offender: "".to_string(),
            description: "Missing publish date".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_forbidden_tag() -> Result<(), String> {
        let test_doc = setup_test_wip_page();
        let test_path = Path::new("wip.html");

        let res = check_forbidden_tags(&test_path, &test_doc);

        let expected_err = Err(CheckError::ContentError {
            path: "wip.html".to_string(),
            offender: "wip".to_string(),
            description: "Forbidden tag".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_forbidden_files() -> Result<(), String> {
        let test_dir = TempDir::new().expect("could not create temp dir");
        let unpublished_dir = test_dir.path().join("unpublished");

        create_dir(&unpublished_dir).expect("failed to create dir");
        let forbidden_file_path = unpublished_dir.join("fobidden.html");
        File::create(&forbidden_file_path).expect("failed to create file");

        let res = check_for_forbidden_files(&forbidden_file_path);

        let expected_err = Err(CheckError::ForbiddenFile {
            path: test_dir
                .path()
                .join("unpublished/fobidden.html")
                .display()
                .to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_correct_file_passes() -> Result<(), String> {
        let test_doc = setup_test_correct_page();
        let test_dir = TempDir::new().expect("could not create temp dir");
        let page_path = test_dir.path().join("page.html");
        let mut f = File::create(&page_path).expect("failed to create file");
        f.write_all(test_doc.root_element().html().as_bytes()).expect("failed to write file contents");

        let res = check_html_file(&page_path);
        assert!(res.is_ok(), "{:?}",res);
        Ok(())
    }
}
