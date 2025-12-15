use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub os: String,
    pub arch: String,
    pub working_dir: PathBuf,
    pub env_vars: HashMap<String, String>,
    pub config: ProjectConfig,
}

impl ExecutionContext {
    pub fn from_current_env() -> Self {
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let env_vars: HashMap<String, String> = std::env::vars().collect();

        Self {
            os,
            arch,
            working_dir,
            env_vars,
            config: ProjectConfig::default(),
        }
    }

    pub fn with_config(mut self, config: ProjectConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(name.into(), value.into());
        self
    }

    pub fn with_working_dir(mut self, path: PathBuf) -> Self {
        self.working_dir = path;
        self
    }

    pub fn resolve_env(&self, name: &str) -> Option<&str> {
        self.env_vars.get(name).map(|s| s.as_str())
    }

    pub fn resolve_config_path(&self, key: &str) -> Option<String> {
        self.config.paths.get(key).map(|m| {
            let path = match self.os.as_str() {
                "linux" => m.linux.as_deref().unwrap_or(&m.default),
                "macos" => m.macos.as_deref().unwrap_or(&m.default),
                "windows" => m.windows.as_deref().unwrap_or(&m.default),
                _ => &m.default,
            };
            self.expand_env_in_string(path)
        })
    }

    pub fn resolve_config_var(&self, key: &str) -> Option<String> {
        self.config.variables.get(key).map(|v| self.expand_env_in_string(v))
    }

    fn expand_env_in_string(&self, s: &str) -> String {
        let mut result = s.to_string();
        for (key, value) in &self.env_vars {
            let pattern = format!("${}", key);
            result = result.replace(&pattern, value);
            let pattern_braces = format!("${{{}}}", key);
            result = result.replace(&pattern_braces, value);
        }
        result
    }

    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.os);
        hasher.update(&self.arch);
        hasher.update(self.working_dir.to_string_lossy().as_bytes());

        let mut sorted_env: Vec<_> = self.env_vars.iter().collect();
        sorted_env.sort_by_key(|(k, _)| *k);
        for (k, v) in sorted_env {
            hasher.update(k);
            hasher.update(v);
        }

        let mut sorted_paths: Vec<_> = self.config.paths.iter().collect();
        sorted_paths.sort_by_key(|(k, _)| *k);
        for (k, v) in sorted_paths {
            hasher.update(k);
            hasher.update(&v.default);
        }

        let mut sorted_vars: Vec<_> = self.config.variables.iter().collect();
        sorted_vars.sort_by_key(|(k, _)| *k);
        for (k, v) in sorted_vars {
            hasher.update(k);
            hasher.update(v);
        }

        format!("{:x}", hasher.finalize())
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::from_current_env()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub paths: HashMap<String, PathMapping>,
    pub variables: HashMap<String, String>,
    pub hosts: HashMap<String, Vec<String>>,
}

impl ProjectConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(mut self, key: impl Into<String>, mapping: PathMapping) -> Self {
        self.paths.insert(key.into(), mapping);
        self
    }

    pub fn with_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    pub fn with_hosts(mut self, key: impl Into<String>, hosts: Vec<String>) -> Self {
        self.hosts.insert(key.into(), hosts);
        self
    }

    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn load(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        Self::from_toml(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathMapping {
    pub default: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<String>,
}

impl PathMapping {
    pub fn new(default: impl Into<String>) -> Self {
        Self {
            default: default.into(),
            linux: None,
            macos: None,
            windows: None,
        }
    }

    pub fn with_linux(mut self, path: impl Into<String>) -> Self {
        self.linux = Some(path.into());
        self
    }

    pub fn with_macos(mut self, path: impl Into<String>) -> Self {
        self.macos = Some(path.into());
        self
    }

    pub fn with_windows(mut self, path: impl Into<String>) -> Self {
        self.windows = Some(path.into());
        self
    }
}

#[derive(Debug, Clone)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_from_env() {
        let ctx = ExecutionContext::from_current_env();
        assert!(!ctx.os.is_empty());
        assert!(!ctx.arch.is_empty());
    }

    #[test]
    fn test_env_resolution() {
        let ctx = ExecutionContext::from_current_env()
            .with_env("MY_VAR", "my_value");

        assert_eq!(ctx.resolve_env("MY_VAR"), Some("my_value"));
        assert_eq!(ctx.resolve_env("NONEXISTENT"), None);
    }

    #[test]
    fn test_config_path_resolution() {
        let config = ProjectConfig::new()
            .with_path("config", PathMapping::new("/default/config")
                .with_linux("/etc/myapp")
                .with_macos("$HOME/Library/Application Support/myapp"));

        let mut ctx = ExecutionContext::from_current_env()
            .with_config(config)
            .with_env("HOME", "/Users/test");

        ctx.os = "macos".to_string();

        let resolved = ctx.resolve_config_path("config");
        assert_eq!(resolved, Some("/Users/test/Library/Application Support/myapp".to_string()));
    }

    #[test]
    fn test_config_variable_resolution() {
        let config = ProjectConfig::new()
            .with_variable("api_url", "https://api.example.com");

        let ctx = ExecutionContext::from_current_env()
            .with_config(config);

        assert_eq!(ctx.resolve_config_var("api_url"), Some("https://api.example.com".to_string()));
    }

    #[test]
    fn test_env_expansion() {
        let ctx = ExecutionContext::from_current_env()
            .with_env("USER", "alice")
            .with_env("HOME", "/home/alice");

        let expanded = ctx.expand_env_in_string("$HOME/.config/$USER/app");
        assert_eq!(expanded, "/home/alice/.config/alice/app");
    }

    #[test]
    fn test_context_hash() {
        let ctx1 = ExecutionContext::from_current_env()
            .with_env("TEST", "value1");
        let ctx2 = ExecutionContext::from_current_env()
            .with_env("TEST", "value2");

        assert_ne!(ctx1.compute_hash(), ctx2.compute_hash());
    }

    #[test]
    fn test_config_from_toml() {
        let toml = r#"
[paths.config]
default = "/default/config"
linux = "/etc/myapp"
macos = "$HOME/Library/myapp"

[variables]
api_url = "https://api.example.com"
debug = "true"

[hosts]
web = ["web1.example.com", "web2.example.com"]
"#;

        let config = ProjectConfig::from_toml(toml).unwrap();
        assert!(config.paths.contains_key("config"));
        assert_eq!(config.variables.get("api_url"), Some(&"https://api.example.com".to_string()));
        assert_eq!(config.hosts.get("web").map(|h| h.len()), Some(2));
    }
}
