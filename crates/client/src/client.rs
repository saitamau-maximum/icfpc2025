use anyhow::Result;
use async_trait::async_trait;
use icfpc2025_common::{
    AedificiumClient, ExploreRequest, ExploreResponse, GuessRequest, GuessResponse, Map,
    SelectRequest, SelectResponse,
};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::env;

pub struct AedificiumRemoteClient {
    id: String,
    client: Client,
    base_url: String,
    debug: bool,
}

fn parse_bool(value: String) -> bool {
    value.to_lowercase() == "true"
}

impl AedificiumRemoteClient {
    pub fn new(id: String) -> Self {
        Self {
            client: Client::new(),
            base_url: "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com".to_string(),
            id,
            debug: parse_bool(env::var("AEDIFICIUM_CLIENT_DEBUG").unwrap_or("false".to_string())),
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
}
#[async_trait]
impl AedificiumClient for AedificiumRemoteClient {
    async fn select(&self, problem_name: String) -> Result<SelectResponse> {
        let data = SelectRequest {
            id: self.id.clone(),
            problem_name,
        };
        self.request("/select", &data).await
    }

    async fn explore(&mut self, plans: Vec<String>) -> Result<ExploreResponse> {
        let data = ExploreRequest {
            id: self.id.clone(),
            plans,
        };
        self.request("/explore", &data).await
    }

    async fn guess(&self, data: Map) -> Result<GuessResponse> {
        let data = GuessRequest {
            id: self.id.clone(),
            map: data,
        };
        self.request("/guess", &data).await
    }
}
