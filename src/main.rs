use std::{borrow::Cow, str::FromStr};

use reqwest::header::{self, HeaderValue, InvalidHeaderValue};
use thiserror::Error;

#[derive(Debug)]
struct Data {
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
struct MediumClient<'a> {
    client: reqwest::Client,
    cookie: Cow<'a, str>,
}

#[derive(Debug, Error)]
enum ClientError {
    #[error("failed to fecth url")]
    FetchFailed(reqwest::Error),

    #[error("failed to parse data")]
    ParseError(reqwest::Error),

    #[error("failed to insert header: {0}")]
    InsertHeaderFailed(InvalidHeaderValue),

    #[error("failed to build client")]
    BuildError(reqwest::Error),
}

impl<'a> MediumClient<'a> {
    fn new(cookie: &'a str) -> Result<Self, ClientError> {
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

    async fn fetch(&self, url: &str) -> Result<Data, ClientError> {
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
}

#[tokio::main]
async fn main() {
    let cookie = r#"uid=1f4ef4ec3d88; sid=1:l17QxZDJ+77kzz9TCbsyWzGhy9sQLuCYulwUNpS8YZWA7R90lZDHutwneJSMz+Tg;"#;
    let client = MediumClient::new(cookie).unwrap();
    let url = "https://medium.com/towards-data-science/how-to-grow-your-career-without-feeling-stuck-c7fe96881a44";
    let output = client.fetch(url).await.unwrap();

    println!("{:#?}", output);
}