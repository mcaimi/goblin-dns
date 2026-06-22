use reqwest::Client as HttpClient;
use serde::Serialize;

use crate::models::*;

pub struct GoblinClient {
    base_url: String,
    http: HttpClient,
}

impl GoblinClient {
    pub fn new(base_url: &str) -> Self {
        GoblinClient {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: HttpClient::new(),
        }
    }

    async fn parse_api_response_if_present<T>(
        resp: reqwest::Response,
        body_required: bool,
    ) -> Result<ApiResponse<T>, GoblinError>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        let bytes = resp.bytes().await.map_err(|e| {
            GoblinError::HttpError(format!("Failed to read response: {}", e))
        })?;

        match body_required {
            false => if bytes.is_empty() {
                    return Ok(ApiResponse {
                        success: true,
                        data: T::default(),
                    });
                },
            true => if bytes.is_empty() {
                    return Err(GoblinError::HttpError(
                        "Unexpected empty response body".to_string(),
                    ));
                }
        }

        let json: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
            GoblinError::HttpError(format!("Failed to parse response: {}", e))
        })?;

        if json["success"] == true {
            Ok(ApiResponse {
                success: true,
                data: if json["data"].is_null() || json["data"] == serde_json::Value::Object(serde_json::Map::new()) {
                    T::default()
                } else {
                    serde_json::from_value(json["data"].clone()).map_err(|e| {
                        GoblinError::HttpError(format!("Failed to parse response: {}", e))
                    })?
                },
            })
        } else {
            println!("{}", "NONOK");
            Err(GoblinError::HttpError("Non-success response".to_string()))
        }
    }

    async fn get<T>(&self, path: &str) -> Result<ApiResponse<T>, GoblinError>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        let url = format!("{}/{}", self.base_url, path);
        let resp = self.http.get(&url).send().await.map_err(|e| {
            GoblinError::HttpError(format!("Failed to send request to {}: {}", url, e))
        })?;

        if !resp.status().is_success() {
            return Err(GoblinError::HttpError(format!(
                "Request to {} failed with status {}",
                url,
                resp.status()
            )));
        }

        Self::parse_api_response_if_present(resp, false).await
    }

    async fn post<B, T>(&self, path: &str, body: &B) -> Result<ApiResponse<T>, GoblinError>
    where
        B: Serialize,
        T: serde::de::DeserializeOwned + Default,
    {
        let url = format!("{}/{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| GoblinError::HttpError(format!("Failed to send request: {}", e)))?;

        if !resp.status().is_success() {
            return Err(GoblinError::HttpError(format!(
                "Request to {} failed with status {}",
                url,
                resp.status()
            )));
        }

        Self::parse_api_response_if_present(resp, true).await
    }

    async fn put(&self, path: &str, body: &serde_json::Value) -> Result<ApiResponse<serde_json::Value>, GoblinError> {
        let url = format!("{}/{}", self.base_url, path);
        let resp = self
            .http
            .put(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| GoblinError::HttpError(format!("Failed to send request: {}", e)))?;

        if !resp.status().is_success() {
            return Err(GoblinError::HttpError(format!(
                "Request to {} failed with status {}",
                url,
                resp.status()
            )));
        }

        Self::parse_api_response_if_present(resp, true).await
    }

    async fn patch(&self, path: &str) -> Result<ApiResponse<serde_json::Value>, GoblinError> {
        let url = format!("{}/{}", self.base_url, path);
        let resp = self
            .http
            .patch(&url)
            .send()
            .await
            .map_err(|e| GoblinError::HttpError(format!("Failed to send request: {}", e)))?;

        if !resp.status().is_success() {
            return Err(GoblinError::HttpError(format!(
                "Request to {} failed with status {}",
                url,
                resp.status()
            )));
        }

        Self::parse_api_response_if_present(resp, true).await
    }

    async fn delete(&self, path: &str) -> Result<ApiResponse<serde_json::Value>, GoblinError> {
        let url = format!("{}/{}", self.base_url, path);
        let resp = self
            .http
            .delete(&url)
            .send()
            .await
            .map_err(|e| GoblinError::HttpError(format!("Failed to send request: {}", e)))?;

        if !resp.status().is_success() {
            return Err(GoblinError::HttpError(format!(
                "Request to {} failed with status {}",
                url,
                resp.status()
            )));
        }

        Self::parse_api_response_if_present(resp, true).await
    }

    pub async fn health_check(&self) -> Result<(), GoblinError> {
        let url = format!("{}/{}", self.base_url, API_PATH_HEALTH);
        let resp = self.http.get(&url).send().await.map_err(|e| {
            GoblinError::HttpError(format!("Failed to send request to {}: {}", url, e))
        })?;

        if !resp.status().is_success() {
            return Err(GoblinError::HttpError(format!(
                "Request to {} failed with status {}",
                url,
                resp.status()
            )));
        }

        Ok(())
    }

    pub async fn list_resolvers(&self) -> Result<ApiResponse<Vec<Resolver>>, GoblinError> {
        self.get(API_PATH_RESOLVERS_LIST).await
    }

    pub async fn get_resolver(&self, id: u64) -> Result<ApiResponse<Resolver>, GoblinError> {
        self.get(&format!("{}{}", API_PATH_RESOLVERS_PREFIX, id)).await
    }

    pub async fn create_resolver(
        &self,
        ip_address: &str,
        domain: &str,
    ) -> Result<ApiResponse<serde_json::Value>, GoblinError> {
        let body = serde_json::json!({
            "ip_address": ip_address,
            "domain": domain,
        });
        self.post(API_PATH_RESOLVERS_LIST, &body).await
    }

    pub async fn update_resolver(
        &self,
        id: u64,
        ip_address: Option<&str>,
        domain: Option<&str>,
    ) -> Result<ApiResponse<serde_json::Value>, GoblinError> {
        let mut obj = serde_json::Map::new();
        if let Some(ip) = ip_address {
            obj.insert("ip_address".to_string(), serde_json::Value::String(ip.to_string()));
        }
        if let Some(d) = domain {
            obj.insert("domain".to_string(), serde_json::Value::String(d.to_string()));
        }
        self.put(&format!("{}{}", API_PATH_RESOLVERS_PREFIX, id), &serde_json::Value::Object(obj))
            .await
    }

    pub async fn delete_resolver(&self, id: u64) -> Result<ApiResponse<serde_json::Value>, GoblinError> {
        self.delete(&format!("{}{}", API_PATH_RESOLVERS_PREFIX, id)).await
    }

    pub async fn list_rules(
        &self,
    ) -> Result<ApiResponse<Vec<Rule>>, GoblinError> {
        self.get(API_PATH_RULES_LIST).await
    }

    pub async fn get_rule(&self, id: u64) -> Result<ApiResponse<Rule>, GoblinError> {
        self.get(&format!("{}{}", API_PATH_RULES_PREFIX, id)).await
    }

    pub async fn create_rule(
        &self,
        domain: &str,
        target_ip: &str,
        client_filter: Option<&str>,
        active: Option<bool>,
    ) -> Result<ApiResponse<serde_json::Value>, GoblinError> {
        let body = serde_json::json!({
            "domain": domain,
            "target_ip": target_ip,
            "client_filter": client_filter,
            "active": active.unwrap_or(true),
        });
        self.post(API_PATH_RULES_LIST, &body).await
    }

    pub async fn update_rule(
        &self,
        id: u64,
        domain: Option<&str>,
        target_ip: Option<&str>,
        client_filter: Option<Option<&str>>,
        active: Option<bool>,
    ) -> Result<ApiResponse<serde_json::Value>, GoblinError> {
        let mut obj = serde_json::Map::new();
        if let Some(d) = domain {
            obj.insert("domain".to_string(), serde_json::Value::String(d.to_string()));
        }
        if let Some(ip) = target_ip {
            obj.insert("target_ip".to_string(), serde_json::Value::String(ip.to_string()));
        }
        if let Some(cf) = client_filter {
            match cf {
                Some(val) => obj.insert("client_filter".to_string(), serde_json::Value::String(val.to_string())),
                None => obj.insert("client_filter".to_string(), serde_json::Value::Null),
            };
        }
        if let Some(a) = active {
            obj.insert("active".to_string(), serde_json::Value::Bool(a));
        }
        self.put(&format!("{}{}", API_PATH_RULES_PREFIX, id), &serde_json::Value::Object(obj))
            .await
    }

    pub async fn toggle_rule(&self, id: u64) -> Result<ApiResponse<serde_json::Value>, GoblinError> {
        self.patch(&format!("{}{}/toggle", API_PATH_RULES_PREFIX, id)).await
    }

    pub async fn delete_rule(&self, id: u64) -> Result<ApiResponse<serde_json::Value>, GoblinError> {
        self.delete(&format!("{}{}", API_PATH_RULES_PREFIX, id)).await
    }

    pub async fn list_stats(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<ApiResponse<Vec<StatEntry>>, GoblinError> {
        let url = format!(
            "{}/{}?limit={}&offset={}",
            self.base_url, API_PATH_STATS, limit, offset
        );
        
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| GoblinError::HttpError(format!("Failed to send request: {}", e)))?;

        if !resp.status().is_success() {
            return Err(GoblinError::HttpError(format!(
                "Request to {} failed with status {}",
                url,
                resp.status()
            )));
        }

        Self::parse_api_response_if_present(resp, false).await
    }

    pub async fn stats_for_rule(
        &self,
        rule_id: u64,
    ) -> Result<ApiResponse<Vec<StatEntry>>, GoblinError> {
        self.get(&format!("{}{}", API_PATH_STATS_RULES, rule_id))
            .await
    }

    pub async fn stats_summary(&self) -> Result<ApiResponse<StatsSummary>, GoblinError> {
        self.get(API_PATH_STATS_SUMMARY).await
    }
}

const API_PATH_HEALTH: &str = "api/v1/health";
const API_PATH_RESOLVERS_PREFIX: &str = "api/v1/resolvers/";
const API_PATH_RESOLVERS_LIST: &str = "api/v1/resolvers";
const API_PATH_RULES_PREFIX: &str = "api/v1/rules/";
const API_PATH_RULES_LIST: &str = "api/v1/rules";
const API_PATH_STATS: &str = "api/v1/stats";
const API_PATH_STATS_SUMMARY: &str = "api/v1/stats/summary";
const API_PATH_STATS_RULES: &str = "api/v1/stats/rules/";
