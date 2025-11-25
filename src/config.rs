// src/config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hyrule_server: String,
    pub auth_token: Option<String>,
    pub username: Option<String>,
    pub default_private: bool,
    pub use_tor: bool,
    pub tor_proxy: String,
    pub verify_ssl: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hyrule_server: "https://hyrule4e3tu7pfdkvvca43senvgvgisi6einpe3d3kpidlk3uyjf7lqd.onion/".to_string(),
            auth_token: None,
            username: None,
            default_private: false,
            use_tor: false,
            tor_proxy: "socks5://127.0.0.1:9050".to_string(),
            verify_ssl: true,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        
        let content = fs::read_to_string(&config_path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
    
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        Ok(config_dir.join("triforge").join("config.toml"))
    }
    
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "server" | "hyrule_server" => {
                self.hyrule_server = value.to_string();
            }
            "username" => {
                self.username = Some(value.to_string());
            }
            "token" | "auth_token" => {
                self.auth_token = Some(value.to_string());
            }
            "private" | "default_private" => {
                self.default_private = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid boolean value"))?;
            }
            "tor" | "use_tor" => {
                self.use_tor = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid boolean value"))?;
            }
            "proxy" | "tor_proxy" => {
                self.tor_proxy = value.to_string();
            }
            "ssl" | "verify_ssl" => {
                self.verify_ssl = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid boolean value"))?;
            }
            _ => {
                anyhow::bail!("Unknown configuration key: {}", key);
            }
        }
        Ok(())
    }
    
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "server" | "hyrule_server" => Some(self.hyrule_server.clone()),
            "username" => self.username.clone(),
            "token" | "auth_token" => self.auth_token.clone(),
            "private" | "default_private" => Some(self.default_private.to_string()),
            "tor" | "use_tor" => Some(self.use_tor.to_string()),
            "proxy" | "tor_proxy" => Some(self.tor_proxy.clone()),
            "ssl" | "verify_ssl" => Some(self.verify_ssl.to_string()),
            _ => None,
        }
    }
    
    pub fn check_tor_available(&self) -> bool {
        use std::net::TcpStream;
        use std::time::Duration;
        
        if !self.use_tor {
            return false;
        }
        
        let proxy_url = self.tor_proxy.trim_start_matches("socks5://");
        
        TcpStream::connect_timeout(
            &proxy_url.parse().unwrap_or_else(|_| "127.0.0.1:9050".parse().unwrap()),
            Duration::from_secs(2)
        ).is_ok()
    }
}
