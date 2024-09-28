use super::{parse_url, print_resp};
use crate::Result;
use clap::Args;
use reqwest::Client;

#[derive(Args, Debug)]
pub struct Get {
    #[arg(value_parser = parse_url)]
    url: String,
}

pub async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}
