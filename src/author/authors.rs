use anyhow::Result;
use hex;
use rain_metadata::meta::RainMetaDocumentV1Item;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    payload: Vec<u8>,
    magic_number: String,
}

async fn fetch_subgraph_dt(url: &str, query: &str) -> Result<serde_json::Value> {
    let client = Client::new();
    let req = client
        .post(url)
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
        return Err(anyhow::anyhow!("Error(s) occurred: {:?}", errors));
    }
    Ok(data)
}

pub async fn get_authors(manager: &str) -> Result<Vec<String>> {
    dbg!(&manager);

    let query = r#"
        query {
          metaV1S {
            meta
          }
        }
    "#;

    // Ensure the URL is using HTTPS for secure communication
    let url = env::var("ADDRESSES_SUBGRAPH_URL").expect("FETCH_URL not set");
    if !url.starts_with("https://") {
        return Err(anyhow::anyhow!("Invalid URL: Must use HTTPS"));
    }

    let res = get_data(&url, query).await?;

    let mut addresses: Vec<String> = Vec::new();
    if let Some(meta_v1s) = res["data"]["metaV1S"].as_array() {
        for item in meta_v1s {
            if let Some(meta_value) = item["meta"].as_str() {
                if meta_value.len() < 18 {
                    // Avoid any out-of-bounds access
                    return Err(anyhow::anyhow!("Invalid meta data: too short"));
                }

                let extracted_substring = &meta_value[18..]; // Remove rain meta magic_number

                // Ensure that the decoded string length matches the expected length
                let bytes_array_meta = match hex::decode(extracted_substring) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to decode hex string: {:?}", e));
                    }
                };

                // CBOR decode and validate
                match RainMetaDocumentV1Item::cbor_decode(&bytes_array_meta) {
                    Ok(cbor_decoded) => {
                        let payload = &cbor_decoded[0].payload;

                        // Ensure that the payload is of the correct length
                        if payload.is_empty() || payload[0] != 1 {
                            return Err(anyhow::anyhow!("Invalid payload structure"));
                        }

                        let address_str: String = hex::encode(payload);
                        let modified_address = format!("0x{}", &address_str[2..]);

                        // Validate address length and format before pushing
                        if modified_address.len() == 42 {
                            addresses.push(modified_address);
                        } else {
                            return Err(anyhow::anyhow!("Invalid address format"));
                        }
                    }
                    Err(err) => {
                        tracing::error!(
                            "Error decoding CBOR for item: {:?}, error: {:?}",
                            item,
                            err
                        );
                        continue;
                    }
                }
            }
        }
    }
    Ok(addresses)
}
