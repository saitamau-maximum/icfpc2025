use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::env;

use crate::types::*;

pub struct AedificiumClient {
    id: String,
    client: Client,
    base_url: String,
    debug: bool,
}

impl AedificiumClient {
    pub fn new(id: String) -> Self {
        Self {
            client: Client::new(),
            base_url: "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com".to_string(),
            id,
            debug: env::var("AEDIFICIUM_CLIENT_DEBUG").is_ok(),
        }
    }

    async fn request<T, R>(&self, endpoint: &str, data: &T) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, endpoint);

        if self.debug {
            println!("=== [DEBUG] AedificiumClient Request ===");
            println!("{}", serde_json::to_string_pretty(data)?);
            println!("========================================");
        }

        let response = self.client.post(&url).json(data).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP {}: {}",
                response.status(),
                response.text().await?
            ));
        }

        let result = response.json::<R>().await?;
        Ok(result)
    }

    pub async fn select(&self, problem_name: String) -> Result<SelectResponse> {
        let data = SelectRequest {
            id: self.id.clone(),
            problem_name,
        };
        self.request("/select", &data).await
    }

    pub async fn explore(&self, plans: Vec<String>) -> Result<ExploreResponse> {
        let data = ExploreRequest {
            id: self.id.clone(),
            plans,
        };
        self.request("/explore", &data).await
    }

    pub async fn guess(&self, data: Map) -> Result<GuessResponse> {
        let data = GuessRequest {
            id: self.id.clone(),
            map: data,
        };
        self.request("/guess", &data).await
    }
}
