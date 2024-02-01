pub mod authors {
    use anyhow::{Result, anyhow};
    use reqwest::Client;
    use serde_cbor::from_slice;

    async fn fetch_subgraph_dt(url: &str, query: &str) -> Result<serde_json::Value> {
        let client = Client::new();
        let req = client.post(url)
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "query": query }).to_string())
            .send()
            .await?;
        
        let text = req.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    async fn get_data(url: &str, query: &str) -> Result<serde_json::Value> {
        let data = fetch_subgraph_dt(url, query).await?;
        if let Some(errors) = data.get("errors") {
            println!("{:?}", errors);
        }
        Ok(data)
    }

pub fn cbor_decode(data: &str) -> Result<Vec<u8>> {
    let trimmed_data = data.trim_start_matches(|c: char| !c.is_ascii_digit());
    let decoded: Vec<u8> = from_slice(trimmed_data.as_bytes())?;
    Ok(decoded)
}

pub async fn get_authors() -> Result<Vec<u8>> {
    let query = r#"
        query {
          metaV1S {
            meta
          }
        }
    "#;

    let url = "https://api.thegraph.com/subgraphs/name/ninokeldishvili/rain-metaboard";
    let res = get_data(url, query).await?;
    
    if let Some(meta) = res.get("data")
        .and_then(|data| data.get("metaV1S"))
        .and_then(|meta_v1s| meta_v1s.get(0))
        .and_then(|first_meta_v1| first_meta_v1.get("meta")) {
        
        let accounts: Vec<u8> = cbor_decode(&meta.to_string())?;
        println!("{:?}", accounts);
        Ok(accounts)
    } else {
        Err(anyhow!("Unable to fetch authors"))
    }
}

}

