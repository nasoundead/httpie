use anyhow::{anyhow, Ok, Result};
use clap::{Args, Parser, Subcommand};
use colored::*;
use mime::Mime;
use reqwest::{header, Client, Response, Url};
use std::{collections::HashMap, str::FromStr};

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    method: Method,
}

#[derive(Subcommand, Debug)]
enum Method {
    Get(Get),
    Post(Post),
}

#[derive(Args, Debug)]
struct Get {
    // clap 允许你为每个解析出来的值添加自定义的解析函数，我们这里定义了个 parse_url 检查一下。
    #[arg(value_parser = parse_url)]
    url: String,
}

#[derive(Args, Debug)]
struct Post {
    /// Specify the url you wanna request to.
    #[arg(value_parser = parse_url)]
    url: String,
    /// Set the request body.
    /// Examples:
    ///     headers:
    ///         header1:value1
    ///     params:
    ///         key1=value1
    #[arg(value_parser = parse_url)]
    body: Vec<KvPair>,
}

pub fn parse_url(s: &str) -> Result<String> {
    // 这里可以直接使用了 str.parse::<Url>() 或 str.parse()，因为 reqwest::Url 实现了 FromStr
    // let _url = s.parse::<Url>()?;
    let _url: Url = s.parse()?;
    Ok(s.into())
}

#[derive(Debug, PartialEq, Clone)]
pub struct KvPair {
    pub key: String,
    pub value: String,
}

/// 当我们实现 FromStr trait 后，可以用 str.parse() 方法将字符串解析成 KvPair
/// FromStr 是 Rust 标准库定义的 trait，实现它之后，就可以调用字符串的 parse() 泛型函数，很方便地处理字符串到某个类型的转换了。
impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self {
            key: (parts.next().ok_or_else(err)?).to_string(),
            value: (parts.next().ok_or_else(err)?).to_string(),
        })
    }
}

pub fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.key, &pair.value);
    }

    let resp = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);

    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }
    print!("\n");
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            // println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
            syntect_print(jsonxf::pretty_print(body).unwrap())
        }
        _ => println!("{}", body),
    }
}

fn syntect_print(s: String) {
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
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    // let client = Client::new();
    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWERED-BY", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);
    let client = Client::builder().default_headers(headers).build()?;

    let result = match opts.method {
        Method::Get(ref args) => get(client, args).await?,
        Method::Post(ref args) => post(client, args).await?,
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        assert!(parse_url("abc").is_err());
        assert!(parse_url("http://abc.xyz").is_ok());
        assert!(parse_url("https://httpbin.org/post").is_ok());
    }

    #[test]
    fn test_parse_kv_pair() {
        assert!(parse_kv_pair("a").is_err());
        assert_eq!(
            parse_kv_pair("a=1").unwrap(),
            KvPair {
                key: "a".into(),
                value: "1".into()
            }
        );
        assert_eq!(
            parse_kv_pair("b=").unwrap(),
            KvPair {
                key: "b".into(),
                value: "".into()
            }
        );
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
