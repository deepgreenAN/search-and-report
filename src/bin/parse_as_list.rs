use scraper::{Html, Selector};
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("row.html")?;
    let mut html_string = String::new();

    file.read_to_string(&mut html_string)?;

    let document = Html::parse_document(&html_string);

    let body_container_selector = Selector::parse(r#"div[class^=Tweet_bodyContainer]"#)?;
    let author_name_selector = Selector::parse(r#"span[class^=Tweet_authorName]"#)?;
    let content_selector = Selector::parse(r#"div[class^=Tweet_body]"#)?;
    let datetime_selector = Selector::parse(r#"time[class^=Tweet_time] > a"#)?;

    for body_container in document.select(&body_container_selector) {
        let author_name = body_container.select(&author_name_selector).next().unwrap();
        let datetime = body_container.select(&datetime_selector).next().unwrap();
        let content = body_container.select(&content_selector).next().unwrap();

        let mut content_buffer = String::new();

        for content_text in content.text() {
            content_buffer.push_str(content_text);
        }

        println!(
            "author: {:?}, datetime: {:?}, content: {:?}",
            author_name.inner_html(),
            datetime.inner_html(),
            content_buffer
        );
    }

    Ok(())
}
