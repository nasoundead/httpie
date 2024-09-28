use super::parse_url;
use super::print_resp;
use crate::Error;
use crate::Result;
use clap::Args;
use reqwest::Client;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Args, Debug)]
pub struct Post {
    #[arg(value_parser = parse_url)]
    url: String,
    /// Set the request body.
    ///     params:
    ///         key1=value1
    #[arg(value_parser = parse_kv_pair)]
    body: Vec<KvPair>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct KvPair {
    pub key: String,
    pub value: String,
}

impl FromStr for KvPair {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split("=");
        let err = || format!("Failed to parse {}", s);

        Ok(Self {
            key: parts.next().ok_or_else(err)?.to_string(),
            value: parts.next().ok_or_else(err)?.to_string(),
        })
    }
}

pub fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

pub async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.key, &pair.value);
    }

    let resp = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
