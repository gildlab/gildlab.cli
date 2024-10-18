use anyhow::Result;
use hex;
use rain_metadata::meta::RainMetaDocumentV1Item;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    payload: Vec<u8>,
    magic_number: String,
}

async fn fetch_subgraph_dt(url: &str, query: &str, variables: Value) -> Result<Value> {
    let client = Client::new();

    let req_body = serde_json::json!({
        "query": query,
        "variables": variables
    });

    let req = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(req_body.to_string())
        .send()
        .await?;
    let text = req.text().await?;
    Ok(serde_json::from_str(&text)?)
}

async fn get_data(url: &str, query: &str, variables: Value) -> Result<Value> {
    let data = fetch_subgraph_dt(url, query, variables).await?;
    if let Some(errors) = data.get("errors") {
        return Err(anyhow::anyhow!("Error(s) occurred: {:?}", errors));
    }
    Ok(data)
}

pub async fn get_authors(manager: &str, url: &str) -> Result<Vec<String>> {
    // Convert manager to lowercase for subgraph
    let manager_lowercase = manager.to_lowercase();

    let query = r#"
        query MyQuery($sender: String!) {
            metaV1S(where: { sender: $sender }) {
                meta
                sender
            }
        }
    "#;

    let variables = json!({
       "sender": manager_lowercase,
    });

    let res = get_data(&url, query, variables).await?;

    let mut address_actions: HashMap<String, u8> = HashMap::new();

    if let Some(meta_v1s) = res["data"]["metaV1S"].as_array() {
        for item in meta_v1s {
            // Filter is made by query parameter, so the result data should already be
            // filtered by manager address.
            // Adding this filter because of test mainly
            if let Some(sender) = item["sender"].as_str() {
                if sender != manager {
                    continue; // Skip non-manager entries
                }
            }
            if let Some(meta_value) = item["meta"].as_str() {
                if meta_value.len() < 18 {
                    // Avoid any out-of-bounds access
                    tracing::error!("Invalid meta data: too short");
                    continue;
                }

                let extracted_substring = &meta_value[18..]; // Remove rain meta magic_number

                // Ensure that the decoded string length matches the expected length
                let bytes_array_meta = match hex::decode(extracted_substring) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        tracing::error!("Failed to decode hex string: {:?}", e);
                        continue;
                    }
                };

                // CBOR decode and validate
                match RainMetaDocumentV1Item::cbor_decode(&bytes_array_meta) {
                    Ok(cbor_decoded) => {
                        let payload = &cbor_decoded[0].payload;

                        // Ensure that the payload is of the correct length
                        if payload.is_empty() {
                            tracing::error!("Invalid payload structure: {:?}", item);
                            continue;
                        }

                        let action_prefix = payload[0]; // 0 for remove, 1 for add
                        let address_str: String = hex::encode(&payload[1..]);
                        let modified_address = format!("0x{}", &address_str);

                        // Validate address length and format before proceeding
                        if modified_address.len() == 42 {
                            // Track the last action for each address
                            address_actions.insert(modified_address, action_prefix);
                        } else {
                            tracing::error!("Invalid Address format");
                            continue;
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

    // Filter addresses that have the last action as 'add' (1)
    let filtered_addresses: Vec<String> = address_actions
        .into_iter()
        .filter_map(
            |(address, action)| {
                if action == 1 {
                    Some(address)
                } else {
                    None
                }
            },
        )
        .collect();
    Ok(filtered_addresses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_authors_with_valid_manager() {
        // Mock subgraph response for multiple addresses
        let mock_subgraph_response = json!({
            "data": {
                "metaV1S": [
                    {
                        "meta": "0xff0a89c674ee7874a3005501c0d477556c25c9d67e1f57245c7453da776b51cf011bffb2637608c09e3802706170706c69636174696f6e2f63626f72", // Mock meta (manager)
                        "sender": "0xc0d477556c25c9d67e1f57245c7453da776b51cf"
                    },
                    {
                        "meta": "0xff0a89c674ee7874a3005501c0d477556c25c9d67e1f57245c7453da776b51cf011bffb2637608c09e3802706170706c69636174696f6e2f63626f72", // Another address's meta
                        "sender": "0x8058ad7c22fdc8788fe4cb1dac15d6e976127324"
                    }
                ]
            }
        });

        // Mock subgraph URL
        let _m = mock("POST", "/")
            .with_header("content-type", "application/json")
            .with_body(mock_subgraph_response.to_string())
            .create();

        // Use mockito URL for testing
        let subgraph_url = &mockito::server_url();

        let manager_address = "0xc0d477556c25c9d67e1f57245c7453da776b51cf";
        let result = get_authors(manager_address, &subgraph_url).await.unwrap();

        assert_eq!(result.len(), 1);
        assert!(result.contains(&manager_address.to_string()));
    }

    #[tokio::test]
    async fn test_get_authors_with_invalid_meta() {
        let mock_subgraph_response = json!({
            "data": {
                "metaV1S": [
                    {
                        "meta": "0000000000000000",
                        "sender": "0xc0d477556c25c9d67e1f57245c7453da776b51cf"
                    },
                    {
                        "meta": "0xff0a89c674ee7874a3005501c0d477556c25c9d67e1f57245c7453da776b51cf011bffb2637608c09e3802706170706c69636174696f6e2f63626f72", // Another address's meta
                        "sender": "0xc0d477556c25c9d67e1f57245c7453da776b51cf"
                    }
                ]
            }
        });

        // Mock subgraph URL
        let _m = mock("POST", "/")
            .with_header("content-type", "application/json")
            .with_body(mock_subgraph_response.to_string())
            .create();

        let subgraph_url = &mockito::server_url();

        let manager_address = "0xc0d477556c25c9d67e1f57245c7453da776b51cf";
        let result = get_authors(manager_address, &subgraph_url).await;
        // Assert that the result is Ok and contains exactly one address
        assert!(result.is_ok(), "Expected Ok result, got Err");

        let addresses = result.unwrap();

        // Assert that the length of the addresses is 1
        assert_eq!(addresses.len(), 1, "Expected 1 valid address, got {:?}", addresses.len());
        // Optionally, you can also assert that the address is the expected one
        let expected_address = "0xc0d477556c25c9d67e1f57245c7453da776b51cf";
        assert_eq!(addresses[0], expected_address, "Unexpected address");
    }
}
