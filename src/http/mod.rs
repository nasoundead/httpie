pub mod get;
pub mod post;

use crate::Result;

use clap::Subcommand;
use colored::*;
use get::Get;
use mime::Mime;
use post::Post;
use reqwest::{header, Response, Url};

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[derive(Subcommand, Debug)]
pub enum Method {
    Get(Get),
    Post(Post),
}
pub fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?;
    Ok(s.into())
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp)?;
    print_headers(&resp)?;

    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body)?;
    Ok(())
}

fn print_status(resp: &Response) -> Result<()> {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
    Ok(())
}

fn print_headers(resp: &Response) -> Result<()> {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }
    print!("\n");
    Ok(())
}

fn print_body(m: Option<Mime>, body: &String) -> Result<()> {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => syntect_print(jsonxf::pretty_print(body)?),
        _ => {
            println!("{}", body);
            Ok(())
        }
    }
}

fn syntect_print(s: String) -> Result<()> {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension("json").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for line in LinesWithEndings::from(&s) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }
    print!("\n");
    Ok(())
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    let header = resp.headers().get(header::CONTENT_TYPE).map(|v| v.to_str());
    match header {
        Some(Ok(v)) => v.parse().ok(),
        _ => None,
    }
}

mod tests {

    #[test]
    fn test_parse_url() {
        use super::parse_url;
        assert!(parse_url("abc").is_err());
        assert!(parse_url("http://abc.xyz").is_ok());
        assert!(parse_url("https://httpbin.org/post").is_ok());
    }

    #[test]
    fn test_pretty_print_unwrap() {
        // assert_eq!(
        //     jsonxf::pretty_print("{\"a\":1,\"b\":2}").unwrap(),
        //     "{\n  \"a\": 1,\n  \"b\": 2\n}"
        // );

        // assert_eq!(jsonxf::pretty_print("hello").unwrap(), "hello");

        let body = r#"{
  "a": 1,
  "b": 2
}"#;
        assert_eq!(jsonxf::pretty_print(body).unwrap(), body);
    }

    #[test]
    fn test_unwrap_or_default() {
        let good_year_from_input = "1909";
        let bad_year_from_input = "190blarg";
        let good_year: u32 = good_year_from_input.parse().unwrap_or_default();
        let bad_year: u32 = bad_year_from_input.parse().unwrap_or_default();

        assert_eq!(1909, good_year);
        assert_eq!(0, bad_year);
    }
}
