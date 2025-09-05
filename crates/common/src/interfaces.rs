use anyhow::Result;
use async_trait::async_trait;

use crate::types::*;

#[async_trait]
pub trait AedificiumClient {
    async fn select(&self, problem_name: String) -> Result<SelectResponse>;
    async fn explore(&self, plans: Vec<String>) -> Result<ExploreResponse>;
    async fn guess(&self, data: Map) -> Result<GuessResponse>;
}
