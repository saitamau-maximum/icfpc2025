use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::types::*;

pub struct AedificiumClient {
    client: Client,
    base_url: String,
}

impl AedificiumClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com".to_string(),
        }
    }

    async fn request<T, R>(&self, endpoint: &str, data: &T) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .json(data)
            .send()
            .await?;

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

    pub async fn register(&self, data: RegisterRequest) -> Result<RegisterResponse> {
        self.request("/register", &data).await
    }

    pub async fn select(&self, data: SelectRequest) -> Result<SelectResponse> {
        self.request("/select", &data).await
    }

    pub async fn explore(&self, data: ExploreRequest) -> Result<ExploreResponse> {
        self.request("/explore", &data).await
    }

    pub async fn guess(&self, data: GuessRequest) -> Result<GuessResponse> {
        self.request("/guess", &data).await
    }
}

impl Default for AedificiumClient {
    fn default() -> Self {
        Self::new()
    }
}