// use anyhow::{anyhow, Ok, Result};
mod error;
mod http;

use crate::error::{Error, Result};
use clap::Parser;
use http::{get::get, post::post, Method};
use reqwest::{header, Client};

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub method: Method,
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
