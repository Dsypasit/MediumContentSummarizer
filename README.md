# MediumContentSummarizer

I made this project for learning rust programming and to solve my problem that I was too lazy to read a medium content.
And I want to make it support ollama if i have enough free time.

## Required

- claude api key
- cargo

## Example

```rust
use summary_medium_post::{AISummary, Claude3agent, MediumClient};

#[tokio::main]
async fn main() {
    let cookie = r#""#;

    // create medium client

    let client = MediumClient::new(cookie).unwrap();
    let url = "https://medium.com/odds-team/unit-tests-%E0%B8%84%E0%B8%B7%E0%B8%AD-executable-document-7fe9e55da4e1";
    //
    // raw html
    let output = client.fetch(url).await.unwrap();

    // medium content
    let medium_content = MediumClient::get_content(output).await.unwrap();

    // create agent
    let claude_agent = Claude3agent::new().unwrap();
    println!("{:?}", claude_agent);

    // summarize
    let summarize = claude_agent.fetch(medium_content).await.unwrap();
    println!("{:#?}", summarize);
}
```

_note_ you have already CLAUDE_API and CLAUDE_URL in your env variable if not then run this and replace value with with your api key

```bash

	CLAUDE_API=VALUE \
		CLAUDE_URL=VALUE \
		cargo run \
```
