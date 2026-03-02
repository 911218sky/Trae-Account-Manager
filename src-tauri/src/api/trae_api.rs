use anyhow::{anyhow, Result};
use reqwest::{header, Client};
use serde_json::json;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use super::types::*;

const API_BASE_US: &str = "https://api-us-east.trae.ai";
const API_BASE_SG: &str = "https://api-sg-central.trae.ai";
const API_BASE_UG: &str = "https://ug-normal.trae.ai";

/// Trae API client for making requests to the Trae service.
pub struct TraeApiClient {
    client: Client,
    cookies: String,
    jwt_token: Option<String>,
    api_base: String,
}

impl TraeApiClient {
    /// Creates a new API client using cookies.
    pub fn new(cookies: &str) -> Result<Self> {
        let client = Client::builder()
            .build()?;

        let cleaned_cookies = cookies
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("")
            .replace("  ", " ");

        let api_base = Self::detect_api_base_from_cookies(&cleaned_cookies);

        Ok(Self {
            client,
            cookies: cleaned_cookies,
            jwt_token: None,
            api_base,
        })
    }

    /// Creates a new API client using a JWT token.
    pub fn new_with_token(token: &str) -> Result<Self> {
        let client = Client::builder()
            .build()?;

        let api_base = API_BASE_SG.to_string();

        Ok(Self {
            client,
            cookies: String::new(),
            jwt_token: Some(token.to_string()),
            api_base,
        })
    }

    /// Detects the API endpoint from cookies.
    fn detect_api_base_from_cookies(cookies: &str) -> String {
        if cookies.contains("store-idc=useast") || cookies.contains("trae-target-idc=useast") {
            API_BASE_US.to_string()
        } else if cookies.contains("store-idc=alisg") || cookies.contains("trae-target-idc=alisg") {
            API_BASE_SG.to_string()
        } else {
            API_BASE_SG.to_string()
        }
    }

    /// Attempts to fetch data from multiple API endpoints.
    #[allow(dead_code)]
    async fn try_api_endpoints<T, F, Fut>(&self, path: &str, request_fn: F) -> Result<T>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let url = format!("{}{}", self.api_base, path);
        match request_fn(url).await {
            Ok(result) => return Ok(result),
            Err(_) => {}
        }

        let other_base = if self.api_base == API_BASE_SG {
            API_BASE_US
        } else {
            API_BASE_SG
        };

        let url = format!("{}{}", other_base, path);
        request_fn(url).await
    }

    /// Builds request headers using only the JWT token.
    fn build_headers_token_only(&self) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse()?);
        headers.insert(header::ACCEPT, "application/json, text/plain, */*".parse()?);
        headers.insert(header::ORIGIN, "https://www.trae.ai".parse()?);
        headers.insert(header::REFERER, "https://www.trae.ai/".parse()?);
        headers.insert(
            header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".parse()?,
        );

        if let Some(token) = &self.jwt_token {
            let auth_value = header::HeaderValue::from_bytes(
                format!("Cloud-IDE-JWT {}", token).as_bytes()
            ).map_err(|e| anyhow!("Token format error: {}", e))?;
            headers.insert(header::AUTHORIZATION, auth_value);
        }

        Ok(headers)
    }

    /// Retrieves user information from the token by parsing JWT and calling the entitlement API.
    pub async fn get_user_info_by_token(&self) -> Result<TokenUserInfo> {
        let token = self.jwt_token.as_ref().ok_or_else(|| anyhow!("Token not found"))?;
        let jwt_data = Self::parse_jwt_token(token)?;

        let headers = self.build_headers_token_only()?;
        let endpoints = [&self.api_base, API_BASE_SG, API_BASE_US];

        let mut last_error = anyhow!("All API endpoints failed");

        for base in endpoints.iter() {
            let url = format!("{}/trae/api/v1/pay/user_current_entitlement_list", base);

            let response = self
                .client
                .post(&url)
                .headers(headers.clone())
                .json(&json!({"require_usage": true}))
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<EntitlementListResponse>().await {
                        Ok(data) => {
                            let user_id_from_api = data.user_entitlement_pack_list
                                .first()
                                .map(|p| p.entitlement_base_info.user_id.clone())
                                .unwrap_or_else(|| jwt_data.user_id.clone());

                            let user_detail = self.get_user_info_with_token().await.ok();

                            return Ok(TokenUserInfo {
                                user_id: user_id_from_api,
                                tenant_id: jwt_data.tenant_id,
                                screen_name: user_detail.as_ref().map(|u| u.screen_name.clone()),
                                avatar_url: user_detail.as_ref().and_then(|u| if u.avatar_url.is_empty() { None } else { Some(u.avatar_url.clone()) }),
                                email: user_detail.as_ref().and_then(|u| u.non_plain_text_email.clone()),
                            });
                        }
                        Err(e) => {
                            last_error = anyhow!("Failed to parse response: {}", e);
                        }
                    }
                }
                Ok(resp) => {
                    last_error = anyhow!("API returned error: {}", resp.status());
                }
                Err(e) => {
                    last_error = anyhow!("Request failed: {}", e);
                }
            }
        }

        Err(last_error)
    }

    /// Calls the GetUserInfo API using the JWT token.
    async fn get_user_info_with_token(&self) -> Result<UserInfoResult> {
        let url = format!("{}/cloudide/api/v3/trae/GetUserInfo", API_BASE_UG);
        let headers = self.build_headers_token_only()?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&json!({"IfWebPage": true}))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get user info: {}", response.status()));
        }

        let data: GetUserInfoResponse = response.json().await?;
        Ok(data.result)
    }

    /// Parses JWT token to extract user information.
    fn parse_jwt_token(token: &str) -> Result<JwtPayload> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid JWT token format"));
        }

        let payload_b64 = parts[1];
        let padding = (4 - payload_b64.len() % 4) % 4;
        let padded = format!("{}{}", payload_b64, "=".repeat(padding));
        let standard_b64 = padded.replace('-', "+").replace('_', "/");

        let payload_bytes = BASE64.decode(&standard_b64)
            .map_err(|e| anyhow!("Failed to decode JWT payload: {}", e))?;

        let payload_str = String::from_utf8(payload_bytes)
            .map_err(|e| anyhow!("JWT payload is not valid UTF-8: {}", e))?;

        let payload: JwtPayloadRaw = serde_json::from_str(&payload_str)
            .map_err(|e| anyhow!("Failed to parse JWT payload: {}", e))?;

        Ok(JwtPayload {
            user_id: payload.data.id,
            tenant_id: payload.data.tenant_id,
        })
    }

    /// Builds request headers with optional authentication.
    fn build_headers(&self, with_auth: bool) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse()?);
        headers.insert(header::ACCEPT, "application/json, text/plain, */*".parse()?);

        let cookie_value = header::HeaderValue::from_bytes(self.cookies.as_bytes())
            .map_err(|e| anyhow!("Cookie format error: {}", e))?;
        headers.insert(header::COOKIE, cookie_value);

        headers.insert(header::ORIGIN, "https://www.trae.ai".parse()?);
        headers.insert(header::REFERER, "https://www.trae.ai/".parse()?);
        headers.insert(
            header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".parse()?,
        );

        if with_auth {
            if let Some(token) = &self.jwt_token {
                let auth_value = header::HeaderValue::from_bytes(
                    format!("Cloud-IDE-JWT {}", token).as_bytes()
                ).map_err(|e| anyhow!("Token format error: {}", e))?;
                headers.insert(header::AUTHORIZATION, auth_value);
            }
        }

        Ok(headers)
    }

    /// Retrieves the user token.
    pub async fn get_user_token(&mut self) -> Result<UserTokenResult> {
        let url = format!("{}/cloudide/api/v3/common/GetUserToken", self.api_base);
        let headers = self.build_headers(false)?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get token: {}", response.status()));
        }

        let data: GetUserTokenResponse = response.json().await?;
        self.jwt_token = Some(data.result.token.clone());
        Ok(data.result)
    }

    /// Retrieves user information.
    pub async fn get_user_info(&self) -> Result<UserInfoResult> {
        let url = format!("{}/cloudide/api/v3/trae/GetUserInfo", API_BASE_UG);
        let headers = self.build_headers(false)?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&json!({"IfWebPage": true}))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get user info: {}", response.status()));
        }

        let data: GetUserInfoResponse = response.json().await?;
        Ok(data.result)
    }

    /// Retrieves user quota and usage information.
    pub async fn get_entitlement_list(&self) -> Result<EntitlementListResponse> {
        let url = format!("{}/trae/api/v1/pay/user_current_entitlement_list", self.api_base);
        let headers = self.build_headers(true)?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&json!({"require_usage": true}))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get quota info: {}", response.status()));
        }

        let data: EntitlementListResponse = response.json().await?;
        Ok(data)
    }

    /// Queries usage records.
    pub async fn query_usage(
        &self,
        start_time: i64,
        end_time: i64,
        page_size: i32,
        page_num: i32,
    ) -> Result<UsageQueryResponse> {
        let url = format!(
            "{}/trae/api/v1/pay/query_user_usage_group_by_session",
            self.api_base
        );
        let headers = self.build_headers(true)?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&json!({
                "start_time": start_time,
                "end_time": end_time,
                "page_size": page_size,
                "page_num": page_num
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to query usage records: {}", response.status()));
        }

        let data: UsageQueryResponse = response.json().await?;
        Ok(data)
    }

    /// Retrieves a summary of usage information.
    pub async fn get_usage_summary(&mut self) -> Result<UsageSummary> {
        if self.jwt_token.is_none() {
            self.get_user_token().await?;
        }

        let entitlements = self.get_entitlement_list().await?;
        Self::parse_entitlements_to_summary(entitlements)
    }

    /// Retrieves usage summary using JWT token.
    pub async fn get_usage_summary_by_token(&self) -> Result<UsageSummary> {
        let headers = self.build_headers_token_only()?;
        let endpoints = [&self.api_base, API_BASE_SG, API_BASE_US];

        let mut last_error = anyhow!("All API endpoints failed");

        for base in endpoints.iter() {
            let url = format!("{}/trae/api/v1/pay/user_current_entitlement_list", base);
            log::debug!("Trying API endpoint: {}", url);

            let response = self
                .client
                .post(&url)
                .headers(headers.clone())
                .json(&json!({"require_usage": true}))
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let response_text = resp.text().await?;
                    log::debug!("API Response from {}: {}", base, response_text);

                    match serde_json::from_str::<EntitlementListResponse>(&response_text) {
                        Ok(entitlements) => {
                            let summary = Self::parse_entitlements_to_summary(entitlements)?;
                            log::debug!("Parsed Summary: fast_request_limit={}, extra_fast_request_limit={}",
                                summary.fast_request_limit, summary.extra_fast_request_limit);
                            return Ok(summary);
                        }
                        Err(e) => {
                            last_error = anyhow!("Failed to parse response: {}", e);
                        }
                    }
                }
                Ok(resp) => {
                    log::debug!("API {} returned error: {}", base, resp.status());
                    last_error = anyhow!("API returned error: {}", resp.status());
                }
                Err(e) => {
                    log::debug!("API {} request failed: {}", base, e);
                    last_error = anyhow!("Request failed: {}", e);
                }
            }
        }

        Err(last_error)
    }

    /// Parses entitlement information into a usage summary.
    fn parse_entitlements_to_summary(entitlements: EntitlementListResponse) -> Result<UsageSummary> {
        let mut summary = UsageSummary::default();

        for pack in entitlements.user_entitlement_pack_list {
            let base = &pack.entitlement_base_info;
            let usage = &pack.usage;
            let quota = &base.quota;

            if base.product_type == 2 {
                summary.extra_fast_request_limit = quota.premium_model_fast_request_limit;
                summary.extra_fast_request_used = usage.premium_model_fast_amount;
                summary.extra_fast_request_left =
                    summary.extra_fast_request_limit as f64 - summary.extra_fast_request_used;
                summary.extra_expire_time = base.end_time;

                if let Some(pkg_extra) = &base.product_extra.package_extra {
                    if pkg_extra.package_source_type == 6 {
                        summary.extra_package_name = "2026 Anniversary Treat".to_string();
                    }
                }
            } else {
                summary.plan_type = if base.product_id == 0 {
                    "Free".to_string()
                } else {
                    "Pro".to_string()
                };
                summary.reset_time = base.end_time;

                summary.fast_request_limit = quota.premium_model_fast_request_limit;
                summary.fast_request_used = usage.premium_model_fast_amount;
                summary.fast_request_left =
                    summary.fast_request_limit as f64 - summary.fast_request_used;

                summary.slow_request_limit = quota.premium_model_slow_request_limit;
                summary.slow_request_used = usage.premium_model_slow_amount;
                summary.slow_request_left =
                    summary.slow_request_limit as f64 - summary.slow_request_used;

                summary.advanced_model_limit = quota.advanced_model_request_limit;
                summary.advanced_model_used = usage.advanced_model_amount;
                summary.advanced_model_left =
                    summary.advanced_model_limit as f64 - summary.advanced_model_used;

                summary.autocomplete_limit = quota.auto_completion_limit;
                summary.autocomplete_used = usage.auto_completion_amount;
                summary.autocomplete_left =
                    summary.autocomplete_limit as f64 - summary.autocomplete_used;
            }
        }

        Ok(summary)
    }

    /// Queries the birthday bonus status.
    pub async fn query_birthday_bonus(&self) -> Result<bool> {
        let url = format!("{}/trae/api/v1/pay/query_birthday_bonus", self.api_base);
        let headers = self.build_headers_token_only()?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to query birthday bonus status: {}", response.status()));
        }

        let data: serde_json::Value = response.json().await?;

        Ok(data["bonus_claimed"].as_bool().unwrap_or(false))
    }

    /// Claims the birthday bonus.
    pub async fn claim_birthday_bonus(&self) -> Result<()> {
        let url = format!("{}/trae/api/v1/pay/claim_birthday_bonus", self.api_base);
        let headers = self.build_headers_token_only()?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to claim birthday bonus: {}", response.status()));
        }

        Ok(())
    }
}
