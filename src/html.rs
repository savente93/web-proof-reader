use scraper::{ElementRef, Html, Selector};
use selectors::attr::CaseSensitivity;
use std::fs::read_to_string;
use std::path::Path;
use std::collections::HashSet;

use crate::CheckError;

use lazy_static::*;
use regex::Regex;
type CheckResult = Result<(), CheckError>;

pub fn check_html_file(path: &Path) -> CheckResult {
    check_for_forbidden_files(path)?;

    let contents = read_to_string(path)?;
    let html = Html::parse_document(&contents);

    check_forbidden_tags(path, &html)?;
    check_for_invalid_publish_dates(path, &html)?;
    check_img_tags_have_alts(path, &html)?;
    check_tags_dont_have_title_attr(path, &html)?;
    check_page_doesnt_disabe_zoom(path, &html)?;
    check_page_has_title(path, &html)?;
    check_page_has_lang_attr(path, &html)?;
    check_page_doesnt_have_positive_tabindex(path, &html)?;
    check_page_doesnt_have_autofocus(path, &html)?;
    check_page_doesnt_have_multiple_h1_elements(path, &html)?;
    check_page_doesnt_have_hrefless_link(path, &html)?;
    check_page_doesnt_have_captionless_figure(path, &html)?;
    check_page_doesnt_have_captionless_table(path, &html)?;
    check_page_doesnt_have_labelless_form_elements(path, &html)?;
    check_page_doesnt_have_autoplay_media(path, &html)?;
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

    for div in document.select(&div_selector) {
        let div_text = &div.text().collect::<Vec<_>>().join("");
        if let Some(publish_date) = extract_iso_date(div_text) {
            if publish_date == "0000-01-01" {
                return Err(CheckError::ContentError {
                    path: path.display().to_string(),
                    offender: publish_date,
                    description: "Forbidden publish date".to_string(),
                });
            }
        }
    }
    Ok(())
}

fn check_img_tags_have_alts(path: &Path, document: &Html) -> CheckResult {
    let img_selector = Selector::parse("img").unwrap();

    for img in document.select(&img_selector) {
        if img.value().attr("alt").is_none() {
            return Err(CheckError::AccessibilityError {
                path: path.display().to_string(),
                offender: img.html(),
                description: "Image tag without alt".to_string(),
            });
        };
    }
    Ok(())
}

fn check_tags_dont_have_title_attr(path: &Path, document: &Html) -> CheckResult {
    let tag_selector = Selector::parse("*").unwrap();

    for tag in document.select(&tag_selector) {
        if tag.value().attr("title").is_some() {
            return Err(CheckError::AccessibilityError {
                path: path.display().to_string(),
                offender: tag.html(),
                description: "Tag in page has title attr".to_string(),
            });
        };
    }
    Ok(())
}
fn check_page_doesnt_disabe_zoom(path: &Path, document: &Html) -> CheckResult {
    // #<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" />
    let meta_selector = Selector::parse("meta").unwrap();
    lazy_static! {
        static ref ZOOM_RE: Regex = Regex::new(r"user-scalable\s*=\s*(no|0)").unwrap();
    }

    for tag in document.select(&meta_selector) {
        match tag.value().attr("content") {
            None => (),
            Some(c) => if ZOOM_RE.is_match(c) {
                return Err(CheckError::AccessibilityError {
                    path: path.display().to_string(),
                    offender: "".to_string(),
                    description: "Page disables zoom".to_string(),
                })
            } else {
                ()
            }
        };
    }
    Ok(())
}
fn check_page_has_title(path: &Path, document: &Html) -> CheckResult {
    let head_selector = Selector::parse("head").unwrap();
    let title_selector = Selector::parse("title").unwrap();

    //we asume the html is valid and thus has exactly 1 head tag
    let head_section = document.select(&head_selector).next().unwrap();
    let title_tag = head_section.select(&title_selector).next();

    match title_tag {
        Some(_) => Ok(()),
        None => Err(CheckError::AccessibilityError {
            path: path.display().to_string(),
            offender: "".to_string(),
            description: "Page is missing a title tag".to_string(),
        }),
    }
}
fn check_page_has_lang_attr(path: &Path, document: &Html) -> CheckResult {
    match document.root_element().value().attr("lang") {
        Some(_) => Ok(()),
        None => Err(CheckError::AccessibilityError {
            path: path.display().to_string(),
            offender: "".to_string(),
            description: "Page doesn't have lang attribute".to_string(),
        }),
    }
}
fn check_page_doesnt_have_positive_tabindex(path: &Path, document: &Html) -> CheckResult {
    let tag_selector = Selector::parse("*").unwrap();

    for tag in document.select(&tag_selector) {
        match tag.value().attr("tabindex") {
            None => (),
            Some(v) => {
                let tabindex = v.parse::<i32>().unwrap();
                if tabindex > 0 {
                    return Err(CheckError::AccessibilityError {
                        path: path.display().to_string(),
                        offender: tag.html(),
                        description: "Page has tag with prositive tab index".to_string(),
                    });
                }
            }
        };
    }

    Ok(())
}
fn check_page_doesnt_have_autofocus(path: &Path, document: &Html) -> CheckResult {
    let tag_selector = Selector::parse("*").unwrap();

    for tag in document.select(&tag_selector) {
        if tag.value().attr("autofocus").is_some() {
            return Err(CheckError::AccessibilityError {
                path: path.display().to_string(),
                offender: tag.html(),
                description: "Tag in page has autofocus attr".to_string(),
            });
        };
    }
    Ok(())
}

fn check_page_doesnt_have_multiple_h1_elements(path: &Path, document: &Html) -> CheckResult {
    let h1_selector = Selector::parse("h1").unwrap();

    let mut h1_iter = document.select(&h1_selector);
    match h1_iter.next() {
        None => Ok(()),
        Some(_) => match h1_iter.next() {
            None => Ok(()),
            Some(h) => Err(CheckError::AccessibilityError {
                path: path.display().to_string(),
                offender: h.html(),
                description: "Has multiple h1 headings".to_string(),
            }),
        },
    }
}
fn check_page_doesnt_have_hrefless_link(path: &Path, document: &Html) -> CheckResult {
    let link_selector = Selector::parse("a").unwrap();

    for link in document.select(&link_selector) {
        if link.value().attr("href").is_none() {
            return Err(CheckError::AccessibilityError {
                path: path.display().to_string(),
                offender: link.html(),
                description: "Link is missing href attribute".to_string(),
            });
        };
    }
    Ok(())
}

fn check_page_doesnt_have_captionless_table(path: &Path, document: &Html) -> CheckResult {
    let table_selector = Selector::parse("table").unwrap();
    let caption_selector = Selector::parse("caption").unwrap();

    for table in document.select(&table_selector) {
        if table.select(&caption_selector).next().is_none(){
            return Err(CheckError::AccessibilityError {
                path: path.display().to_string(),
                offender: "".to_string(),
                description: "Page contains table without caption".to_string(),
            });
        };
    }
    Ok(())
}
fn check_page_doesnt_have_captionless_figure(path: &Path, document: &Html) -> CheckResult {
    let fig_selector = Selector::parse("figure").unwrap();
    let figcap_selector = Selector::parse("figcaption").unwrap();
    for fig in document.select(&fig_selector) {
        if fig.select(&figcap_selector).next().is_none(){
            return Err(CheckError::AccessibilityError {
                path: path.display().to_string(),
                offender: "".to_string(),
                description: "Page contains figure without caption".to_string(),
            });
        };
    }
    Ok(())
}
fn check_page_doesnt_have_labelless_form_elements(path: &Path, document: &Html) -> CheckResult {
    let form_selector = Selector::parse("form").unwrap();
    let input_selector = Selector::parse("input").unwrap();
    let label_selector = Selector::parse("label").unwrap();

    for form in document.select(&form_selector) {
        
        let input_ids = form.select(&input_selector).filter_map(|input| input.value().id()).collect::<HashSet<_>>();
        let label_ids = form.select(&label_selector).filter_map(|input| input.value().attr("for")).collect::<HashSet<_>>();

        for id in input_ids{
            if !label_ids.contains(id){
                return Err(CheckError::AccessibilityError {
                    path: path.display().to_string(),
                    offender: format!("id=\"{}\"",id),
                    description: "Form element without label attr".to_string(),
                });
            }
        }
    }
    Ok(())
}
fn check_page_doesnt_have_autoplay_media(path: &Path, document: &Html) -> CheckResult {
    let tag_selector = Selector::parse("*").unwrap();

    for tag in document.select(&tag_selector) {
        if tag.value().attr("autoplay").is_some() {
            return Err(CheckError::AccessibilityError {
                path: path.display().to_string(),
                offender: tag.html(),
                description: "Page has media with autoplay enabled".to_string(),
            });
        };
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
    use std::io::prelude::*;
    use tempfile::TempDir;

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

    fn setup_wrong_a11y_page() -> Html {
        let wip_page_contents = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" />
        </head>
        <body>
            <main>
                <article>
                <h3>a h3 that's way before it's time</h3>
                <h1>First title</h1>
                <h1>Second title</h1>
                <header>
                    <h3>A "complete" web page</h3>
                    <div class="date">Published: 2021-04-13</div>
                    <hr>
                </header>
                <section>
                <p> if you enjoy this content, hit the subscribe button!</p>
                <img src="img_girl.jpg"> 
                <a title="a useless link"></a>
                <div tabindex="24"></div>
                <form action="/action_page.php">
                <input autofocus><br><br>
                <input type="radio" name="whatever" id="other" value="other"><br><br>
                </form> 

<figure>
<img src="pic_trulli.jpg" alt="Trulli" style="width:100%">
</figure> 
<table>
<tr>
  <th>Month</th>
  <th>Savings</th>
</tr>
<tr>
  <td>January</td>
  <td>$100</td>
</tr>
</table> 
                <audio autoplay></audio> 
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
        f.write_all(test_doc.root_element().html().as_bytes())
            .expect("failed to write file contents");

        let res = check_html_file(&page_path);
        assert!(res.is_ok(), "{:?}", res);
        Ok(())
    }

    #[test]
    fn test_discovers_img_without_alt() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_img_tags_have_alts(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "<img src=\"img_girl.jpg\">".to_string(),
            description: "Image tag without alt".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_tag_with_title_attr() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_tags_dont_have_title_attr(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "<a title=\"a useless link\"></a>".to_string(),
            description: "Tag in page has title attr".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_disabled_zoom() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_doesnt_disabe_zoom(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "".to_string(),
            description: "Page disables zoom".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_page_without_title() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_has_title(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "".to_string(),
            description: "Page is missing a title tag".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_page_without_lang_attr() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_has_lang_attr(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "".to_string(),
            description: "Page doesn't have lang attribute".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_page_with_positive_tabindex() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_doesnt_have_positive_tabindex(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "<div tabindex=\"24\"></div>".to_string(),
            description: "Page has tag with prositive tab index".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_page_with_autofocus_attr() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_doesnt_have_autofocus(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "<input autofocus=\"\">".to_string(),
            description: "Tag in page has autofocus attr".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_page_with_multiple_h1_elements() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_doesnt_have_multiple_h1_elements(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "<h1>Second title</h1>".to_string(),
            description: "Has multiple h1 headings".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_page_with_hrefless_link() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_doesnt_have_hrefless_link(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "<a title=\"a useless link\"></a>".to_string(),
            description: "Link is missing href attribute".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_page_with_captionless_table() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_doesnt_have_captionless_table(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "".to_string(),
            description: "Page contains table without caption".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_page_with_captionless_figure() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_doesnt_have_captionless_figure(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "".to_string(),
            description: "Page contains figure without caption".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_page_with_labelless_form() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_doesnt_have_labelless_form_elements(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "id=\"other\"".to_string(),
            description: "Form element without label attr".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }

    #[test]
    fn test_discovers_page_with_autoplay_media() -> Result<(), String> {
        let test_doc = setup_wrong_a11y_page();
        let test_path = Path::new("wip.html");

        let res = check_page_doesnt_have_autoplay_media(&test_path, &test_doc);

        let expected_err = Err(CheckError::AccessibilityError {
            path: "wip.html".to_string(),
            offender: "<audio autoplay=\"\"></audio>".to_string(),
            description: "Page has media with autoplay enabled".to_string(),
        });

        assert_eq!(res, expected_err);
        Ok(())
    }
}
