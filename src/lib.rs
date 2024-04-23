use std::{
    borrow::Cow,
    env::{self, VarError},
    str::FromStr,
};

use regex::Regex;
use reqwest::header::{self, HeaderMap, HeaderValue, InvalidHeaderValue};
use serde::{de::Error, Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug)]
pub struct Data {
    url: String,
    body: String,
    status: String,
}

impl Data {
    fn new(url: String, body: String, status: String) -> Self {
        Self { url, body, status }
    }
}

#[derive(Debug)]
pub struct MediumClient<'a> {
    pub client: reqwest::Client,
    cookie: Cow<'a, str>,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to fecth url")]
    FetchFailed(reqwest::Error),

    #[error("failed to parse data")]
    ParseError(reqwest::Error),

    #[error("failed to insert header: {0}")]
    InsertHeaderFailed(InvalidHeaderValue),

    #[error("failed to build client")]
    BuildError(reqwest::Error),

    #[error("failed to use regex")]
    RegexError(regex::Error),

    #[error("Not found")]
    MissMatch,
}

impl<'a> MediumClient<'a> {
    pub fn new(cookie: &'a str) -> Result<Self, ClientError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(cookie).map_err(ClientError::InsertHeaderFailed)?,
        );
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_str("https://medium.com").map_err(ClientError::InsertHeaderFailed)?,
        );
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_str(
                "Mozilla/5.0 (X11; Linux x86_64; rv:124.0) Gecko/20100101 Firefox/124.0",
            )
            .map_err(ClientError::InsertHeaderFailed)?,
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(ClientError::BuildError)?;
        Ok(Self {
            client,
            cookie: Cow::Borrowed(cookie),
        })
    }

    pub async fn fetch(&self, url: &str) -> Result<Data, ClientError> {
        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(ClientError::FetchFailed)
            .unwrap();

        let status_code = res.status().as_str().to_owned();
        let raw_data = res.text().await.map_err(ClientError::ParseError).unwrap();

        let result = Data::new(url.to_owned(), raw_data.to_owned(), status_code);

        Ok(result)
    }

    pub async fn get_content(data: Data) -> Result<String, ClientError> {
        let text = r#"text":\s*"((?:[^"\\]|\\.)*)"#;
        let re = Regex::new(text).map_err(ClientError::RegexError).unwrap();
        let mut m = vec![];
        for (_, [out]) in re.captures_iter(&data.body).map(|c| c.extract()) {
            m.push(out);
        }
        let result = m.join(" ");
        Ok(result)
    }
}

#[derive(Debug, Error)]
pub enum AISummaryError {
    #[error("failed to fetch summary from agent")]
    FetchFailed(ClientError),

    #[error("No api key")]
    NoAPIKey(VarError),

    #[error("No api url")]
    NoAPIURL(VarError),
}

pub trait AISummary<T> {
    async fn fetch(&self, content: String) -> Result<T, AISummaryError>;
    fn build_body(&self, content: String) -> serde_json::Value;
}

#[derive(Debug)]
pub struct Claude3agent {
    apikey: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claude3resposeContent {
    text: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Claude3respose {
    content: Vec<Claude3resposeContent>,
    id: String,
    model: String,
}

impl AISummary<Claude3respose> for Claude3agent {
    async fn fetch(&self, content: String) -> Result<Claude3respose, AISummaryError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            header::HeaderValue::from_str(&self.apikey.clone())
                .map_err(|err| AISummaryError::FetchFailed(ClientError::InsertHeaderFailed(err)))?,
        );

        headers.insert(
            "anthropic-version",
            header::HeaderValue::from_str("2023-06-01")
                .map_err(|err| AISummaryError::FetchFailed(ClientError::InsertHeaderFailed(err)))?,
        );

        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_str("application/json")
                .map_err(|err| AISummaryError::FetchFailed(ClientError::InsertHeaderFailed(err)))?,
        );

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .map_err(|err| AISummaryError::FetchFailed(ClientError::FetchFailed(err)))?;

        let body = self.build_body(content);

        let res = client
            .post(&self.url)
            .body(body.to_string())
            .send()
            .await
            .map_err(|err| AISummaryError::FetchFailed(ClientError::FetchFailed(err)))
            .unwrap();

        let result = res
            .json::<Claude3respose>()
            .await
            .map_err(|err| AISummaryError::FetchFailed(ClientError::ParseError(err)))?;
        Ok(result)
    }

    fn build_body(&self, content: String) -> serde_json::Value {
        let data = json!(
        {
        "model": "claude-3-haiku-20240307",
        "system": "can you summarize this as bullet point with english lang",
        "max_tokens": 1024,
        "messages": [
        {
        "role":"user",
        "content": content
        }
        ]
        }
        );
        return data;
    }
}

impl Claude3agent {
    pub fn new() -> Result<Self, AISummaryError> {
        let apikey = env::var("CLAUDE_API")
            .map_err(AISummaryError::NoAPIKey)
            .unwrap();
        let url = env::var("CLAUDE_URL")
            .map_err(AISummaryError::NoAPIURL)
            .unwrap();
        Ok(Self { apikey, url })
    }
}

struct OllamaAgent {}

impl OllamaAgent {
    fn new() -> Self {
        Self {}
    }
}
