// src/api.rs - API client with Tor support
use crate::config::AppConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Request/Response types (keeping existing types)
#[derive(Debug, Serialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub description: Option<String>,
    pub storage_tier: String,
    pub is_private: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateRepoResponse {
    pub repo_hash: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct RepoMetadata {
    pub repo_hash: String,
    pub name: String,
    pub description: Option<String>,
    pub size: i64,
    pub replica_count: i64,
    #[serde(default)]
    pub nodes: Vec<NodeInfo>,
    pub health_status: String,
    #[serde(default)]
    pub star_count: i64,
    #[serde(default)]
    pub fork_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListObjectsResponse {
    pub objects: Vec<String>,
    pub count: usize,
}

#[derive(Debug, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub address: String,
    pub port: i32,
    pub is_anchor: bool,
    pub storage_capacity: i64,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub storage_used: i64,
    pub storage_quota: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct UploadObjectRequest {
    pub object_id: String,
    pub object_type: String,
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadObjectResponse {
    pub success: bool,
    pub object_id: String,
}

#[derive(Debug, Serialize)]
pub struct BatchUploadRequest {
    pub objects: Vec<UploadObjectRequest>,
}

#[derive(Debug, Deserialize)]
pub struct BatchUploadResponse {
    pub uploaded: usize,
    pub failed: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateRefRequest {
    pub ref_name: String,
    pub commit_id: String,
}

#[derive(Debug, Serialize)]
pub struct ForkRequest {
    pub new_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForkResponse {
    pub forked_hash: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AddTagsRequest {
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct NetworkStats {
    pub total_repos: i64,
    pub total_nodes: i64,
    pub total_users: i64,
    pub total_storage: i64,
    pub anchor_nodes: i64,
    pub p2p_nodes: i64,
    pub avg_replication: f64,
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub repo_hash: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub star_count: i64,
    #[serde(default)]
    pub fork_count: i64,
    #[serde(default)]
    pub size: i64,
}

pub struct ApiClient {
    config: AppConfig,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(config: AppConfig) -> Self {
        let client = Self::build_client(&config);
        Self { config, client }
    }

    /// Build HTTP client with Tor support
    fn build_client(config: &AppConfig) -> reqwest::Client {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(60)) // Longer timeout for Tor
            .connect_timeout(Duration::from_secs(30));

        // Configure Tor proxy if enabled
        if config.use_tor {
            if let Ok(proxy) = reqwest::Proxy::all(&config.tor_proxy) {
                builder = builder.proxy(proxy);
            }
        }

        // Disable SSL verification for onion services
        if !config.verify_ssl {
            builder = builder.danger_accept_invalid_certs(true);
        }

        builder.build().expect("Failed to build HTTP client")
    }

    // Authentication
    pub async fn login(&self, username: &str, password: &str) -> anyhow::Result<LoginResponse> {
        let url = format!("{}/api/auth/login", self.config.hyrule_server);
        let req = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self.client.post(&url).json(&req).send().await?;
        let status = response.status();
        let body_text = response.text().await?;

        if !status.is_success() {
            anyhow::bail!("Login failed ({}): {}", status, body_text);
        }

        serde_json::from_str(&body_text)
            .map_err(|e| anyhow::anyhow!("Parse error: {}. Body: {}", e, body_text))
    }

    pub async fn signup(&self, username: &str, password: &str) -> anyhow::Result<LoginResponse> {
        let url = format!("{}/api/auth/signup", self.config.hyrule_server);
        let req = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self.client.post(&url).json(&req).send().await?;
        let status = response.status();
        let body_text = response.text().await?;

        if !status.is_success() {
            anyhow::bail!("Signup failed ({}): {}", status, body_text);
        }

        serde_json::from_str(&body_text)
            .map_err(|e| anyhow::anyhow!("Parse error: {}. Body: {}", e, body_text))
    }

    // Repository operations
    pub async fn create_repo(&self, req: CreateRepoRequest) -> anyhow::Result<CreateRepoResponse> {
        let token = self
            .config
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!("{}/api/repos", self.config.hyrule_server);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to create repository ({}): {}", status, body);
        }

        Ok(response.json().await?)
    }

    pub async fn get_repo(&self, repo_hash: &str) -> anyhow::Result<RepoMetadata> {
        let url = format!("{}/api/repos/{}", self.config.hyrule_server, repo_hash);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Repository not found: {}", repo_hash);
        }

        Ok(response.json().await?)
    }

    pub async fn list_objects(&self, repo_hash: &str) -> anyhow::Result<ListObjectsResponse> {
        let url = format!(
            "{}/api/repos/{}/objects",
            self.config.hyrule_server, repo_hash
        );
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list objects: {}", response.status());
        }

        Ok(response.json().await?)
    }

    pub async fn batch_upload_objects(
        &self,
        repo_hash: &str,
        objects: Vec<UploadObjectRequest>,
    ) -> anyhow::Result<BatchUploadResponse> {
        let token = self
            .config
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!(
            "{}/api/repos/{}/objects/batch",
            self.config.hyrule_server, repo_hash
        );
        let req = BatchUploadRequest { objects };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to upload objects ({}): {}", status, body);
        }

        let body_text = response.text().await?;
        Ok(serde_json::from_str::<BatchUploadResponse>(&body_text)?)
    }

    pub async fn download_object(
        &self,
        repo_hash: &str,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        let url = format!(
            "{}/api/repos/{}/objects/{}",
            self.config.hyrule_server, repo_hash, object_id
        );
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download object: {}", object_id);
        }

        Ok(response.bytes().await?.to_vec())
    }

    pub async fn update_ref(
        &self,
        repo_hash: &str,
        ref_name: &str,
        commit_id: &str,
    ) -> anyhow::Result<()> {
        let token = self
            .config
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!("{}/api/repos/{}/refs", self.config.hyrule_server, repo_hash);
        let req = UpdateRefRequest {
            ref_name: ref_name.to_string(),
            commit_id: commit_id.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to update ref");
        }

        Ok(())
    }

    pub async fn get_ref(&self, repo_hash: &str, ref_name: &str) -> anyhow::Result<String> {
        let encoded_ref = urlencoding::encode(ref_name);
        let url = format!(
            "{}/api/repos/{}/refs/{}",
            self.config.hyrule_server, repo_hash, encoded_ref
        );
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Ref not found: {}", ref_name);
        }

        Ok(response.text().await?)
    }

    pub async fn delete_repo(&self, repo_hash: &str) -> anyhow::Result<()> {
        let token = self
            .config
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let url = format!("{}/api/repos/{}", self.config.hyrule_server, repo_hash);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to delete repository");
        }

        Ok(())
    }

    // Additional methods (star, unstar, pin, unpin, fork, tags, search, etc.)
    // ... keeping all existing methods ...
    
    pub async fn star_repo(&self, repo_hash: &str) -> anyhow::Result<()> {
        let token = self.config.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;
        let url = format!("{}/api/repos/{}/star", self.config.hyrule_server, repo_hash);
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to star ({}): {}", status, body);
        }
        Ok(())
    }

    pub async fn unstar_repo(&self, repo_hash: &str) -> anyhow::Result<()> {
        let token = self.config.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;
        let url = format!("{}/api/repos/{}/star", self.config.hyrule_server, repo_hash);
        let response = self.client.delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to unstar repository");
        }
        Ok(())
    }

    pub async fn get_starred(&self) -> anyhow::Result<Vec<RepoMetadata>> {
        let token = self.config.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;
        let url = format!("{}/api/repos/starred", self.config.hyrule_server);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to get starred repositories");
        }
        Ok(response.json().await?)
    }

    pub async fn pin_repo(&self, repo_hash: &str) -> anyhow::Result<()> {
        let token = self.config.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;
        let url = format!("{}/api/repos/{}/pin", self.config.hyrule_server, repo_hash);
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to pin repository");
        }
        Ok(())
    }

    pub async fn unpin_repo(&self, repo_hash: &str) -> anyhow::Result<()> {
        let token = self.config.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;
        let url = format!("{}/api/repos/{}/unpin", self.config.hyrule_server, repo_hash);
        let response = self.client.delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to unpin repository");
        }
        Ok(())
    }

    pub async fn get_pinned(&self) -> anyhow::Result<Vec<RepoMetadata>> {
        let token = self.config.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;
        let url = format!("{}/api/repos/pinned", self.config.hyrule_server);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to get pinned repositories");
        }
        Ok(response.json().await?)
    }

    pub async fn fork_repo(
        &self,
        repo_hash: &str,
        new_name: &str,
        description: Option<&str>,
    ) -> anyhow::Result<ForkResponse> {
        let token = self.config.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;
        let url = format!("{}/api/repos/{}/fork", self.config.hyrule_server, repo_hash);
        let req = ForkRequest {
            new_name: new_name.to_string(),
            description: description.map(|s| s.to_string()),
        };
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&req)
            .send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to fork repository ({}): {}", status, body);
        }
        Ok(response.json().await?)
    }

    pub async fn add_tags(&self, repo_hash: &str, tags: Vec<String>) -> anyhow::Result<()> {
        let token = self.config.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;
        let url = format!("{}/api/repos/{}/tags", self.config.hyrule_server, repo_hash);
        let req = AddTagsRequest { tags };
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&req)
            .send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to add tags ({}): {}", status, body);
        }
        Ok(())
    }

    pub async fn get_repo_tags(&self, repo_hash: &str) -> anyhow::Result<Vec<String>> {
        let url = format!("{}/api/repos/{}/tags", self.config.hyrule_server, repo_hash);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to get tags");
        }
        Ok(response.json().await?)
    }

    pub async fn get_all_tags(&self) -> anyhow::Result<Vec<(String, i64)>> {
        let url = format!("{}/api/tags", self.config.hyrule_server);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to get all tags");
        }
        Ok(response.json().await?)
    }

    pub async fn get_repos_by_tag(&self, tag: &str) -> anyhow::Result<Vec<RepoMetadata>> {
        let url = format!("{}/api/tags/{}/repos", self.config.hyrule_server, tag);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to get repositories by tag");
        }
        Ok(response.json().await?)
    }

    pub async fn search_repos(
        &self,
        query: &str,
        tags: Vec<String>,
        user: Option<String>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let mut url = format!(
            "{}/api/repos/search?q={}",
            self.config.hyrule_server,
            urlencoding::encode(query)
        );
        if !tags.is_empty() {
            url.push_str(&format!("&tags={}", tags.join(",")));
        }
        if let Some(u) = user {
            url.push_str(&format!("&user={}", urlencoding::encode(&u)));
        }
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Search failed ({}): {}", status, body);
        }
        let body = response.text().await?;
        match serde_json::from_str::<Vec<SearchResult>>(&body) {
            Ok(results) => Ok(results),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_trending(&self, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
        let url = format!("{}/api/repos/trending?limit={}", self.config.hyrule_server, limit);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to get trending ({}): {}", status, body);
        }
        let body = response.text().await?;
        match serde_json::from_str::<Vec<SearchResult>>(&body) {
            Ok(results) => Ok(results),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_popular(&self, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
        let url = format!("{}/api/repos/popular?limit={}", self.config.hyrule_server, limit);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to get popular ({}): {}", status, body);
        }
        let body = response.text().await?;
        match serde_json::from_str::<Vec<SearchResult>>(&body) {
            Ok(results) => Ok(results),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_network_stats(&self) -> anyhow::Result<NetworkStats> {
        let url = format!("{}/api/stats", self.config.hyrule_server);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to get stats ({}): {}", status, body);
        }
        let body = response.text().await?;
        serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse stats: {}. Body: {}", e, body))
    }

    pub async fn list_nodes(&self) -> anyhow::Result<Vec<NodeInfo>> {
        let url = format!("{}/api/nodes", self.config.hyrule_server);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to list nodes");
        }
        Ok(response.json().await?)
    }

    pub async fn list_user_repos(&self) -> anyhow::Result<Vec<RepoMetadata>> {
        let token = self.config.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;
        let url = format!("{}/api/repos/user", self.config.hyrule_server);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to list repositories");
        }
        Ok(response.json().await?)
    }
}
