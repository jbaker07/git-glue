//! Production-ready adapters for EDR/SIEM/cloud ingest.
//! Features: OAuth2/API key auth, pagination, retries, rate limiting, config-driven mapping.

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

// ============================================================================
// Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub name: String,
    pub api_url: String,
    pub auth: AuthConfig,
    pub pagination: PaginationConfig,
    pub retry: RetryConfig,
    pub field_mapping: FieldMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthConfig {
    Bearer {
        token: String,
    },
    OAuth2 {
        client_id: String,
        client_secret: String,
        token_url: String,
    },
    ApiKey {
        header_name: String,
        api_key: String,
    },
    Basic {
        username: String,
        password: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    pub enabled: bool,
    pub page_size: usize,
    pub max_pages: usize,
    pub cursor_field: Option<String>,
    pub offset_field: Option<String>,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            page_size: 100,
            max_pages: 10,
            cursor_field: None,
            offset_field: Some("offset".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub event_type: String,
    pub timestamp: String,
    pub host: String,
    pub user: String,
    pub severity: Option<String>,
    pub description: Option<String>,
    pub process_name: Option<String>,
    pub process_path: Option<String>,
    pub parent_process: Option<String>,
    pub command_line: Option<String>,
    pub file_path: Option<String>,
    pub network_dest_ip: Option<String>,
    pub network_dest_port: Option<String>,
    pub custom_fields: HashMap<String, String>,
}

// ============================================================================
// Normalized Event
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedEvent {
    pub id: String,
    pub source: String,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub host: String,
    pub user: Option<String>,
    pub severity: Option<String>,
    pub description: Option<String>,
    pub process: Option<ProcessInfo>,
    pub network: Option<NetworkInfo>,
    pub file: Option<FileInfo>,
    pub raw: Value,
    pub custom_fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: Option<String>,
    pub path: Option<String>,
    pub command_line: Option<String>,
    pub parent_name: Option<String>,
    pub pid: Option<u32>,
    pub parent_pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub dest_ip: Option<String>,
    pub dest_port: Option<u16>,
    pub src_ip: Option<String>,
    pub src_port: Option<u16>,
    pub protocol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: Option<String>,
    pub hash_sha256: Option<String>,
    pub hash_md5: Option<String>,
    pub size: Option<u64>,
}

// ============================================================================
// Adapter Trait
// ============================================================================

#[async_trait::async_trait]
pub trait Adapter: Send + Sync {
    fn name(&self) -> &str;
    fn config(&self) -> &AdapterConfig;

    async fn fetch_events(&self) -> Result<Vec<Value>>;
    fn map_event(&self, event: &Value) -> Result<NormalizedEvent>;

    async fn fetch_and_normalize(&self) -> Result<Vec<NormalizedEvent>> {
        let raw_events = self.fetch_events().await?;
        let mut normalized = Vec::with_capacity(raw_events.len());
        for event in &raw_events {
            match self.map_event(event) {
                Ok(n) => normalized.push(n),
                Err(e) => warn!("Failed to normalize event: {}", e),
            }
        }
        info!(
            "{}: Fetched {} events, normalized {}",
            self.name(),
            raw_events.len(),
            normalized.len()
        );
        Ok(normalized)
    }
}

// ============================================================================
// Base Adapter Implementation
// ============================================================================

pub struct BaseAdapter {
    config: AdapterConfig,
    client: Client,
    access_token: tokio::sync::RwLock<Option<String>>,
}

impl BaseAdapter {
    pub fn new(config: AdapterConfig) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
        Ok(Self {
            config,
            client,
            access_token: tokio::sync::RwLock::new(None),
        })
    }

    async fn get_auth_header(&self) -> Result<(String, String)> {
        match &self.config.auth {
            AuthConfig::Bearer { token } => {
                Ok(("Authorization".to_string(), format!("Bearer {}", token)))
            }
            AuthConfig::ApiKey {
                header_name,
                api_key,
            } => Ok((header_name.clone(), api_key.clone())),
            AuthConfig::Basic { username, password } => {
                let encoded = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    format!("{}:{}", username, password),
                );
                Ok(("Authorization".to_string(), format!("Basic {}", encoded)))
            }
            AuthConfig::OAuth2 {
                client_id,
                client_secret,
                token_url,
            } => {
                let mut token = self.access_token.write().await;
                if token.is_none() {
                    let new_token = self
                        .fetch_oauth2_token(client_id, client_secret, token_url)
                        .await?;
                    *token = Some(new_token);
                }
                Ok((
                    "Authorization".to_string(),
                    format!("Bearer {}", token.as_ref().unwrap()),
                ))
            }
        }
    }

    async fn fetch_oauth2_token(
        &self,
        client_id: &str,
        client_secret: &str,
        token_url: &str,
    ) -> Result<String> {
        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];
        let resp = self.client.post(token_url).form(&params).send().await?;
        let json: Value = resp.json().await?;
        json.get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .context("No access_token in OAuth2 response")
    }

    async fn fetch_with_retry(&self, url: &str, body: Option<Value>) -> Result<Value> {
        let retry = &self.config.retry;
        let mut delay = retry.initial_delay_ms;

        for attempt in 0..=retry.max_retries {
            let (header_name, header_value) = self.get_auth_header().await?;

            let mut req = match &body {
                Some(b) => self.client.post(url).json(b),
                None => self.client.get(url),
            };
            req = req.header(&header_name, &header_value);

            match req.send().await {
                Ok(resp) => {
                    match resp.status() {
                        StatusCode::OK | StatusCode::CREATED => {
                            return resp.json().await.context("Failed to parse JSON response");
                        }
                        StatusCode::TOO_MANY_REQUESTS => {
                            warn!("Rate limited, retrying in {}ms", delay);
                        }
                        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                            // Invalidate OAuth2 token
                            *self.access_token.write().await = None;
                            warn!("Auth failed, retrying with fresh token");
                        }
                        status => {
                            error!("Request failed with status {}", status);
                            if attempt == retry.max_retries {
                                anyhow::bail!(
                                    "Request failed after {} retries: {}",
                                    retry.max_retries,
                                    status
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Request error: {}", e);
                    if attempt == retry.max_retries {
                        return Err(e.into());
                    }
                }
            }

            sleep(Duration::from_millis(delay)).await;
            delay =
                ((delay as f64) * retry.backoff_multiplier).min(retry.max_delay_ms as f64) as u64;
        }

        anyhow::bail!("Request failed after all retries")
    }

    async fn fetch_paginated(
        &self,
        base_url: &str,
        events_path: &[&str],
        body_template: Option<Value>,
    ) -> Result<Vec<Value>> {
        let pagination = &self.config.pagination;
        if !pagination.enabled {
            let json = self.fetch_with_retry(base_url, body_template).await?;
            return Ok(extract_nested_array(&json, events_path));
        }

        let mut all_events = Vec::new();
        let mut offset = 0;
        let mut cursor: Option<String> = None;

        for page in 0..pagination.max_pages {
            let mut body = body_template.clone().unwrap_or(serde_json::json!({}));
            if let Some(offset_field) = &pagination.offset_field {
                body[offset_field] = serde_json::json!(offset);
            }
            if let Some(cursor_field) = &pagination.cursor_field {
                if let Some(ref c) = cursor {
                    body[cursor_field] = serde_json::json!(c);
                }
            }
            body["limit"] = serde_json::json!(pagination.page_size);

            let json = self.fetch_with_retry(base_url, Some(body)).await?;
            let events = extract_nested_array(&json, events_path);
            let count = events.len();
            all_events.extend(events);

            debug!("Page {}: fetched {} events", page, count);

            if count < pagination.page_size {
                break;
            }

            offset += pagination.page_size;
            cursor = json
                .get("cursor")
                .or(json.get("next_cursor"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }

        Ok(all_events)
    }

    fn map_field(&self, event: &Value, field_path: &str) -> Option<Value> {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = event;
        for part in parts {
            current = current.get(part)?;
        }
        Some(current.clone())
    }

    fn map_event_with_config(&self, event: &Value) -> Result<NormalizedEvent> {
        let mapping = &self.config.field_mapping;

        let timestamp_str = self
            .map_field(event, &mapping.timestamp)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        let mut custom_fields = HashMap::new();
        for (key, path) in &mapping.custom_fields {
            if let Some(v) = self.map_field(event, path) {
                custom_fields.insert(key.clone(), v);
            }
        }

        Ok(NormalizedEvent {
            id: uuid::Uuid::new_v4().to_string(),
            source: self.config.name.clone(),
            event_type: self
                .map_field(event, &mapping.event_type)
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown".to_string()),
            timestamp,
            host: self
                .map_field(event, &mapping.host)
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown".to_string()),
            user: self
                .map_field(event, &mapping.user)
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            severity: mapping
                .severity
                .as_ref()
                .and_then(|s| self.map_field(event, s))
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            description: mapping
                .description
                .as_ref()
                .and_then(|s| self.map_field(event, s))
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            process: Some(ProcessInfo {
                name: mapping
                    .process_name
                    .as_ref()
                    .and_then(|s| self.map_field(event, s))
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                path: mapping
                    .process_path
                    .as_ref()
                    .and_then(|s| self.map_field(event, s))
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                command_line: mapping
                    .command_line
                    .as_ref()
                    .and_then(|s| self.map_field(event, s))
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                parent_name: mapping
                    .parent_process
                    .as_ref()
                    .and_then(|s| self.map_field(event, s))
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                pid: None,
                parent_pid: None,
            }),
            network: Some(NetworkInfo {
                dest_ip: mapping
                    .network_dest_ip
                    .as_ref()
                    .and_then(|s| self.map_field(event, s))
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                dest_port: mapping
                    .network_dest_port
                    .as_ref()
                    .and_then(|s| self.map_field(event, s))
                    .and_then(|v| v.as_u64().map(|p| p as u16)),
                src_ip: None,
                src_port: None,
                protocol: None,
            }),
            file: Some(FileInfo {
                path: mapping
                    .file_path
                    .as_ref()
                    .and_then(|s| self.map_field(event, s))
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                hash_sha256: None,
                hash_md5: None,
                size: None,
            }),
            raw: event.clone(),
            custom_fields,
        })
    }
}

fn extract_nested_array(json: &Value, path: &[&str]) -> Vec<Value> {
    let mut current = json;
    for part in path {
        match current.get(*part) {
            Some(v) => current = v,
            None => return Vec::new(),
        }
    }
    current.as_array().cloned().unwrap_or_default()
}

// ============================================================================
// CrowdStrike Adapter
// ============================================================================

pub struct CrowdStrikeAdapter {
    base: BaseAdapter,
}

impl CrowdStrikeAdapter {
    pub fn new(api_url: String, client_id: String, client_secret: String) -> Result<Self> {
        let config = AdapterConfig {
            name: "CrowdStrike".to_string(),
            api_url: api_url.clone(),
            auth: AuthConfig::OAuth2 {
                client_id,
                client_secret,
                token_url: format!("{}/oauth2/token", api_url),
            },
            pagination: PaginationConfig {
                enabled: true,
                page_size: 100,
                max_pages: 10,
                cursor_field: Some("offset".to_string()),
                offset_field: None,
            },
            retry: RetryConfig::default(),
            field_mapping: FieldMapping {
                event_type: "behaviors.tactic".to_string(),
                timestamp: "created_timestamp".to_string(),
                host: "device.hostname".to_string(),
                user: "device.local_ip".to_string(),
                severity: Some("max_severity_displayname".to_string()),
                description: Some("behaviors.description".to_string()),
                process_name: Some("behaviors.filename".to_string()),
                process_path: Some("behaviors.filepath".to_string()),
                parent_process: Some(
                    "behaviors.parent_details.parent_process_graph_id".to_string(),
                ),
                command_line: Some("behaviors.cmdline".to_string()),
                file_path: None,
                network_dest_ip: None,
                network_dest_port: None,
                custom_fields: HashMap::new(),
            },
        };
        Ok(Self {
            base: BaseAdapter::new(config)?,
        })
    }
}

#[async_trait::async_trait]
impl Adapter for CrowdStrikeAdapter {
    fn name(&self) -> &str {
        "CrowdStrike"
    }
    fn config(&self) -> &AdapterConfig {
        &self.base.config
    }

    async fn fetch_events(&self) -> Result<Vec<Value>> {
        let url = format!("{}/detects/queries/detects/v1", self.base.config.api_url);
        let ids_json = self.base.fetch_with_retry(&url, None).await?;
        let ids: Vec<String> = ids_json
            .get("resources")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let details_url = format!(
            "{}/detects/entities/summaries/GET/v1",
            self.base.config.api_url
        );
        let body = serde_json::json!({ "ids": ids });
        let details_json = self.base.fetch_with_retry(&details_url, Some(body)).await?;
        Ok(extract_nested_array(&details_json, &["resources"]))
    }

    fn map_event(&self, event: &Value) -> Result<NormalizedEvent> {
        self.base.map_event_with_config(event)
    }
}

// ============================================================================
// SentinelOne Adapter
// ============================================================================

pub struct SentinelOneAdapter {
    base: BaseAdapter,
}

impl SentinelOneAdapter {
    pub fn new(api_url: String, api_token: String) -> Result<Self> {
        let config = AdapterConfig {
            name: "SentinelOne".to_string(),
            api_url,
            auth: AuthConfig::ApiKey {
                header_name: "Authorization".to_string(),
                api_key: format!("ApiToken {}", api_token),
            },
            pagination: PaginationConfig {
                enabled: true,
                page_size: 100,
                max_pages: 10,
                cursor_field: Some("cursor".to_string()),
                offset_field: None,
            },
            retry: RetryConfig::default(),
            field_mapping: FieldMapping {
                event_type: "threatInfo.threatName".to_string(),
                timestamp: "createdAt".to_string(),
                host: "agentRealtimeInfo.agentComputerName".to_string(),
                user: "threatInfo.originatorProcess.user".to_string(),
                severity: Some("threatInfo.confidenceLevel".to_string()),
                description: Some("threatInfo.threatName".to_string()),
                process_name: Some("threatInfo.originatorProcess.name".to_string()),
                process_path: Some("threatInfo.filePath".to_string()),
                parent_process: Some("threatInfo.originatorProcess.parentProcessName".to_string()),
                command_line: Some("threatInfo.originatorProcess.commandLine".to_string()),
                file_path: Some("threatInfo.filePath".to_string()),
                network_dest_ip: None,
                network_dest_port: None,
                custom_fields: HashMap::new(),
            },
        };
        Ok(Self {
            base: BaseAdapter::new(config)?,
        })
    }
}

#[async_trait::async_trait]
impl Adapter for SentinelOneAdapter {
    fn name(&self) -> &str {
        "SentinelOne"
    }
    fn config(&self) -> &AdapterConfig {
        &self.base.config
    }

    async fn fetch_events(&self) -> Result<Vec<Value>> {
        let url = format!("{}/web/api/v2.1/threats", self.base.config.api_url);
        self.base.fetch_paginated(&url, &["data"], None).await
    }

    fn map_event(&self, event: &Value) -> Result<NormalizedEvent> {
        self.base.map_event_with_config(event)
    }
}

// ============================================================================
// Microsoft Defender Adapter
// ============================================================================

pub struct DefenderAdapter {
    base: BaseAdapter,
}

impl DefenderAdapter {
    pub fn new(tenant_id: String, client_id: String, client_secret: String) -> Result<Self> {
        let api_url = "https://api.securitycenter.microsoft.com".to_string();
        let config = AdapterConfig {
            name: "Microsoft Defender".to_string(),
            api_url,
            auth: AuthConfig::OAuth2 {
                client_id,
                client_secret,
                token_url: format!(
                    "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                    tenant_id
                ),
            },
            pagination: PaginationConfig {
                enabled: true,
                page_size: 100,
                max_pages: 10,
                cursor_field: None,
                offset_field: Some("$skip".to_string()),
            },
            retry: RetryConfig::default(),
            field_mapping: FieldMapping {
                event_type: "category".to_string(),
                timestamp: "alertCreationTime".to_string(),
                host: "computerDnsName".to_string(),
                user: "lastEventTime".to_string(),
                severity: Some("severity".to_string()),
                description: Some("title".to_string()),
                process_name: Some("evidence.processName".to_string()),
                process_path: Some("evidence.processPath".to_string()),
                parent_process: Some("evidence.parentProcessName".to_string()),
                command_line: Some("evidence.processCommandLine".to_string()),
                file_path: Some("evidence.filePath".to_string()),
                network_dest_ip: Some("evidence.remoteIP".to_string()),
                network_dest_port: Some("evidence.remotePort".to_string()),
                custom_fields: HashMap::new(),
            },
        };
        Ok(Self {
            base: BaseAdapter::new(config)?,
        })
    }
}

#[async_trait::async_trait]
impl Adapter for DefenderAdapter {
    fn name(&self) -> &str {
        "Microsoft Defender"
    }
    fn config(&self) -> &AdapterConfig {
        &self.base.config
    }

    async fn fetch_events(&self) -> Result<Vec<Value>> {
        let url = format!("{}/api/alerts", self.base.config.api_url);
        self.base.fetch_paginated(&url, &["value"], None).await
    }

    fn map_event(&self, event: &Value) -> Result<NormalizedEvent> {
        self.base.map_event_with_config(event)
    }
}

// ============================================================================
// Palo Alto Cortex XDR Adapter
// ============================================================================

pub struct CortexXDRAdapter {
    base: BaseAdapter,
}

impl CortexXDRAdapter {
    pub fn new(api_url: String, api_key: String, api_key_id: String) -> Result<Self> {
        let config = AdapterConfig {
            name: "Cortex XDR".to_string(),
            api_url,
            auth: AuthConfig::ApiKey {
                header_name: "x-xdr-auth-id".to_string(),
                api_key: format!("{}:{}", api_key_id, api_key),
            },
            pagination: PaginationConfig {
                enabled: true,
                page_size: 100,
                max_pages: 10,
                cursor_field: None,
                offset_field: Some("search_from".to_string()),
            },
            retry: RetryConfig::default(),
            field_mapping: FieldMapping {
                event_type: "alert_name".to_string(),
                timestamp: "detection_timestamp".to_string(),
                host: "host_name".to_string(),
                user: "user_name".to_string(),
                severity: Some("severity".to_string()),
                description: Some("description".to_string()),
                process_name: Some("action_process_image_name".to_string()),
                process_path: Some("action_process_image_path".to_string()),
                parent_process: Some("actor_process_image_name".to_string()),
                command_line: Some("action_process_image_command_line".to_string()),
                file_path: Some("action_file_path".to_string()),
                network_dest_ip: Some("action_remote_ip".to_string()),
                network_dest_port: Some("action_remote_port".to_string()),
                custom_fields: HashMap::new(),
            },
        };
        Ok(Self {
            base: BaseAdapter::new(config)?,
        })
    }
}

#[async_trait::async_trait]
impl Adapter for CortexXDRAdapter {
    fn name(&self) -> &str {
        "Cortex XDR"
    }
    fn config(&self) -> &AdapterConfig {
        &self.base.config
    }

    async fn fetch_events(&self) -> Result<Vec<Value>> {
        let url = format!(
            "{}/public_api/v1/alerts/get_alerts/",
            self.base.config.api_url
        );
        let body = serde_json::json!({
            "request_data": {
                "filters": [],
                "search_from": 0,
                "search_to": 100,
                "sort": { "field": "creation_time", "keyword": "desc" }
            }
        });
        self.base
            .fetch_paginated(&url, &["reply", "alerts"], Some(body))
            .await
    }

    fn map_event(&self, event: &Value) -> Result<NormalizedEvent> {
        self.base.map_event_with_config(event)
    }
}

// ============================================================================
// Carbon Black Adapter
// ============================================================================

pub struct CarbonBlackAdapter {
    base: BaseAdapter,
    org_key: String,
}

impl CarbonBlackAdapter {
    pub fn new(
        api_url: String,
        api_id: String,
        api_secret: String,
        org_key: String,
    ) -> Result<Self> {
        let config = AdapterConfig {
            name: "Carbon Black".to_string(),
            api_url,
            auth: AuthConfig::ApiKey {
                header_name: "X-Auth-Token".to_string(),
                api_key: format!("{}/{}", api_secret, api_id),
            },
            pagination: PaginationConfig::default(),
            retry: RetryConfig::default(),
            field_mapping: FieldMapping {
                event_type: "type".to_string(),
                timestamp: "backend_timestamp".to_string(),
                host: "device_name".to_string(),
                user: "device_username".to_string(),
                severity: Some("severity".to_string()),
                description: Some("reason".to_string()),
                process_name: Some("process_name".to_string()),
                process_path: Some("process_path".to_string()),
                parent_process: Some("parent_name".to_string()),
                command_line: Some("process_cmdline".to_string()),
                file_path: None,
                network_dest_ip: Some("netconn_remote_ip".to_string()),
                network_dest_port: Some("netconn_remote_port".to_string()),
                custom_fields: HashMap::new(),
            },
        };
        Ok(Self {
            base: BaseAdapter::new(config)?,
            org_key,
        })
    }
}

#[async_trait::async_trait]
impl Adapter for CarbonBlackAdapter {
    fn name(&self) -> &str {
        "Carbon Black"
    }
    fn config(&self) -> &AdapterConfig {
        &self.base.config
    }

    async fn fetch_events(&self) -> Result<Vec<Value>> {
        let url = format!(
            "{}/api/alerts/v7/orgs/{}/alerts/_search",
            self.base.config.api_url, self.org_key
        );
        let body = serde_json::json!({
            "rows": 100,
            "start": 0,
            "sort_field": "create_time",
            "sort_order": "DESC"
        });
        self.base
            .fetch_paginated(&url, &["results"], Some(body))
            .await
    }

    fn map_event(&self, event: &Value) -> Result<NormalizedEvent> {
        self.base.map_event_with_config(event)
    }
}

// ============================================================================
// Sophos Adapter
// ============================================================================

pub struct SophosAdapter {
    base: BaseAdapter,
}

impl SophosAdapter {
    pub fn new(api_url: String, client_id: String, client_secret: String) -> Result<Self> {
        let config = AdapterConfig {
            name: "Sophos".to_string(),
            api_url,
            auth: AuthConfig::OAuth2 {
                client_id,
                client_secret,
                token_url: "https://id.sophos.com/api/v2/oauth2/token".to_string(),
            },
            pagination: PaginationConfig::default(),
            retry: RetryConfig::default(),
            field_mapping: FieldMapping {
                event_type: "type".to_string(),
                timestamp: "raisedAt".to_string(),
                host: "managedAgent.name".to_string(),
                user: "person.name".to_string(),
                severity: Some("severity".to_string()),
                description: Some("description".to_string()),
                process_name: Some("data.processName".to_string()),
                process_path: Some("data.processPath".to_string()),
                parent_process: None,
                command_line: Some("data.commandLine".to_string()),
                file_path: Some("data.filePath".to_string()),
                network_dest_ip: Some("data.remoteAddress".to_string()),
                network_dest_port: Some("data.remotePort".to_string()),
                custom_fields: HashMap::new(),
            },
        };
        Ok(Self {
            base: BaseAdapter::new(config)?,
        })
    }
}

#[async_trait::async_trait]
impl Adapter for SophosAdapter {
    fn name(&self) -> &str {
        "Sophos"
    }
    fn config(&self) -> &AdapterConfig {
        &self.base.config
    }

    async fn fetch_events(&self) -> Result<Vec<Value>> {
        let url = format!("{}/siem/v1/alerts", self.base.config.api_url);
        self.base.fetch_paginated(&url, &["items"], None).await
    }

    fn map_event(&self, event: &Value) -> Result<NormalizedEvent> {
        self.base.map_event_with_config(event)
    }
}

// ============================================================================
// Remaining adapters follow same pattern
// ============================================================================

pub struct McAfeeAdapter {
    base: BaseAdapter,
}
pub struct TrendMicroAdapter {
    base: BaseAdapter,
}
pub struct EsetAdapter {
    base: BaseAdapter,
}
pub struct BitdefenderAdapter {
    base: BaseAdapter,
}

impl McAfeeAdapter {
    pub fn new(api_url: String, api_key: String) -> Result<Self> {
        let config = AdapterConfig {
            name: "McAfee".to_string(),
            api_url,
            auth: AuthConfig::Bearer { token: api_key },
            pagination: PaginationConfig::default(),
            retry: RetryConfig::default(),
            field_mapping: FieldMapping {
                event_type: "threatType".to_string(),
                timestamp: "receivedUtc".to_string(),
                host: "machineName".to_string(),
                user: "userName".to_string(),
                severity: Some("severity".to_string()),
                description: Some("threatName".to_string()),
                process_name: Some("processName".to_string()),
                process_path: Some("targetPath".to_string()),
                parent_process: None,
                command_line: None,
                file_path: Some("targetPath".to_string()),
                network_dest_ip: None,
                network_dest_port: None,
                custom_fields: HashMap::new(),
            },
        };
        Ok(Self {
            base: BaseAdapter::new(config)?,
        })
    }
}

impl TrendMicroAdapter {
    pub fn new(api_url: String, api_key: String) -> Result<Self> {
        let config = AdapterConfig {
            name: "Trend Micro".to_string(),
            api_url,
            auth: AuthConfig::ApiKey {
                header_name: "Authorization".to_string(),
                api_key: format!("Bearer {}", api_key),
            },
            pagination: PaginationConfig::default(),
            retry: RetryConfig::default(),
            field_mapping: FieldMapping {
                event_type: "eventType".to_string(),
                timestamp: "eventTime".to_string(),
                host: "endpointName".to_string(),
                user: "accountName".to_string(),
                severity: Some("severity".to_string()),
                description: Some("description".to_string()),
                process_name: Some("processName".to_string()),
                process_path: Some("processFilePath".to_string()),
                parent_process: Some("parentProcessName".to_string()),
                command_line: Some("commandLine".to_string()),
                file_path: Some("objectFilePath".to_string()),
                network_dest_ip: Some("destinationIp".to_string()),
                network_dest_port: Some("destinationPort".to_string()),
                custom_fields: HashMap::new(),
            },
        };
        Ok(Self {
            base: BaseAdapter::new(config)?,
        })
    }
}

impl EsetAdapter {
    pub fn new(api_url: String, username: String, password: String) -> Result<Self> {
        let config = AdapterConfig {
            name: "ESET".to_string(),
            api_url,
            auth: AuthConfig::Basic { username, password },
            pagination: PaginationConfig::default(),
            retry: RetryConfig::default(),
            field_mapping: FieldMapping {
                event_type: "threat_type".to_string(),
                timestamp: "occured".to_string(),
                host: "hostname".to_string(),
                user: "username".to_string(),
                severity: Some("severity".to_string()),
                description: Some("threat_name".to_string()),
                process_name: Some("processname".to_string()),
                process_path: None,
                parent_process: None,
                command_line: None,
                file_path: Some("objecturi".to_string()),
                network_dest_ip: None,
                network_dest_port: None,
                custom_fields: HashMap::new(),
            },
        };
        Ok(Self {
            base: BaseAdapter::new(config)?,
        })
    }
}

impl BitdefenderAdapter {
    pub fn new(api_url: String, api_key: String) -> Result<Self> {
        let config = AdapterConfig {
            name: "Bitdefender".to_string(),
            api_url,
            auth: AuthConfig::Bearer { token: api_key },
            pagination: PaginationConfig::default(),
            retry: RetryConfig::default(),
            field_mapping: FieldMapping {
                event_type: "module".to_string(),
                timestamp: "detected_at".to_string(),
                host: "computer_name".to_string(),
                user: "user".to_string(),
                severity: Some("severity".to_string()),
                description: Some("malware_name".to_string()),
                process_name: Some("process_name".to_string()),
                process_path: Some("process_path".to_string()),
                parent_process: None,
                command_line: Some("process_command_line".to_string()),
                file_path: Some("file_path".to_string()),
                network_dest_ip: None,
                network_dest_port: None,
                custom_fields: HashMap::new(),
            },
        };
        Ok(Self {
            base: BaseAdapter::new(config)?,
        })
    }
}

// Implement Adapter trait for remaining adapters
macro_rules! impl_adapter {
    ($name:ident, $display:expr, $url_path:expr, $events_path:expr) => {
        #[async_trait::async_trait]
        impl Adapter for $name {
            fn name(&self) -> &str {
                $display
            }
            fn config(&self) -> &AdapterConfig {
                &self.base.config
            }

            async fn fetch_events(&self) -> Result<Vec<Value>> {
                let url = format!("{}{}", self.base.config.api_url, $url_path);
                self.base.fetch_paginated(&url, $events_path, None).await
            }

            fn map_event(&self, event: &Value) -> Result<NormalizedEvent> {
                self.base.map_event_with_config(event)
            }
        }
    };
}

impl_adapter!(McAfeeAdapter, "McAfee", "/epo/alerts", &["alerts"]);
impl_adapter!(
    TrendMicroAdapter,
    "Trend Micro",
    "/v3.0/workbench/alerts",
    &["items"]
);
impl_adapter!(EsetAdapter, "ESET", "/v1/detections", &["data"]);
impl_adapter!(
    BitdefenderAdapter,
    "Bitdefender",
    "/v1.0/jsonrpc/push/getEvents",
    &["result", "items"]
);

// ============================================================================
// Adapter Registry
// ============================================================================

pub struct AdapterRegistry {
    adapters: Vec<Box<dyn Adapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    pub fn register(&mut self, adapter: Box<dyn Adapter>) {
        self.adapters.push(adapter);
    }

    pub async fn fetch_all(&self) -> Vec<NormalizedEvent> {
        let mut all_events = Vec::new();
        for adapter in &self.adapters {
            match adapter.fetch_and_normalize().await {
                Ok(events) => all_events.extend(events),
                Err(e) => error!("Failed to fetch from {}: {}", adapter.name(), e),
            }
        }
        all_events
    }

    pub fn adapters(&self) -> &[Box<dyn Adapter>] {
        &self.adapters
    }
}
