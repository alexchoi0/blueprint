use crate::action::Action;
use glob::Pattern;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
pub struct Policy {
    #[serde(default)]
    pub filesystem: FilesystemPolicy,
    #[serde(default)]
    pub network: NetworkPolicy,
    #[serde(default)]
    pub exec: ExecPolicy,
    #[serde(default)]
    pub env: EnvPolicy,
}

#[derive(Debug, Default, Deserialize)]
pub struct FilesystemPolicy {
    #[serde(default)]
    pub allow_read: Vec<String>,
    #[serde(default)]
    pub deny_read: Vec<String>,
    #[serde(default)]
    pub allow_write: Vec<String>,
    #[serde(default)]
    pub deny_write: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct NetworkPolicy {
    #[serde(default)]
    pub allow_http: Vec<String>,
    #[serde(default)]
    pub deny_http: Vec<String>,
    #[serde(default)]
    pub allow_tcp: Vec<String>,
    #[serde(default)]
    pub deny_tcp: Vec<String>,
    #[serde(default)]
    pub allow_udp: Vec<String>,
    #[serde(default)]
    pub deny_udp: Vec<String>,
    #[serde(default)]
    pub allow_unix: Vec<String>,
    #[serde(default)]
    pub deny_unix: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ExecPolicy {
    #[serde(default)]
    pub allow_commands: Vec<String>,
    #[serde(default)]
    pub deny_commands: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct EnvPolicy {
    #[serde(default)]
    pub allow_vars: Vec<String>,
    #[serde(default)]
    pub deny_vars: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny,
    NoMatch,
}

impl Policy {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let policy: Policy = toml::from_str(&content)?;
        Ok(policy)
    }

    pub fn check(&self, action: &Action) -> PolicyDecision {
        match action {
            Action::ReadFile { path } | Action::ListDir { path } => {
                self.check_patterns(path, &self.filesystem.allow_read, &self.filesystem.deny_read)
            }

            Action::WriteFile { path }
            | Action::AppendFile { path }
            | Action::DeleteFile { path }
            | Action::CreateDir { path }
            | Action::DeleteDir { path } => {
                self.check_patterns(path, &self.filesystem.allow_write, &self.filesystem.deny_write)
            }

            Action::CopyFile { src, dst } | Action::MoveFile { src, dst } => {
                let src_decision = self.check_patterns(
                    src,
                    &self.filesystem.allow_read,
                    &self.filesystem.deny_read,
                );
                if src_decision == PolicyDecision::Deny {
                    return PolicyDecision::Deny;
                }

                let dst_decision = self.check_patterns(
                    dst,
                    &self.filesystem.allow_write,
                    &self.filesystem.deny_write,
                );
                if dst_decision == PolicyDecision::Deny {
                    return PolicyDecision::Deny;
                }

                if src_decision == PolicyDecision::Allow && dst_decision == PolicyDecision::Allow {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::NoMatch
                }
            }

            Action::HttpRequest { url, .. } => {
                self.check_patterns(url, &self.network.allow_http, &self.network.deny_http)
            }

            Action::TcpConnect { host, port } | Action::TcpListen { host, port } => {
                let addr = format!("{}:{}", host, port);
                self.check_address_patterns(&addr, &self.network.allow_tcp, &self.network.deny_tcp)
            }

            Action::UdpBind { host, port } | Action::UdpSendTo { host, port } => {
                let addr = format!("{}:{}", host, port);
                self.check_address_patterns(&addr, &self.network.allow_udp, &self.network.deny_udp)
            }

            Action::UnixConnect { path } | Action::UnixListen { path } => {
                self.check_patterns(path, &self.network.allow_unix, &self.network.deny_unix)
            }

            Action::Exec { command, .. } => {
                self.check_command(command, &self.exec.allow_commands, &self.exec.deny_commands)
            }

            Action::EnvGet { name } => {
                self.check_patterns(name, &self.env.allow_vars, &self.env.deny_vars)
            }

            Action::WebhookServe { host, port } => {
                let addr = format!("{}:{}", host, port);
                self.check_address_patterns(&addr, &self.network.allow_tcp, &self.network.deny_tcp)
            }

            Action::WatchFiles { patterns } => {
                for pattern in patterns {
                    let decision = self.check_patterns(
                        pattern,
                        &self.filesystem.allow_read,
                        &self.filesystem.deny_read,
                    );
                    if decision == PolicyDecision::Deny {
                        return PolicyDecision::Deny;
                    }
                }
                PolicyDecision::NoMatch
            }
        }
    }

    fn check_patterns(
        &self,
        value: &str,
        allow_patterns: &[String],
        deny_patterns: &[String],
    ) -> PolicyDecision {
        for pattern in deny_patterns {
            if let Ok(p) = Pattern::new(pattern) {
                if p.matches(value) {
                    return PolicyDecision::Deny;
                }
            }
        }

        for pattern in allow_patterns {
            if let Ok(p) = Pattern::new(pattern) {
                if p.matches(value) {
                    return PolicyDecision::Allow;
                }
            }
        }

        PolicyDecision::NoMatch
    }

    fn check_address_patterns(
        &self,
        addr: &str,
        allow_patterns: &[String],
        deny_patterns: &[String],
    ) -> PolicyDecision {
        for pattern in deny_patterns {
            if self.matches_address_pattern(addr, pattern) {
                return PolicyDecision::Deny;
            }
        }

        for pattern in allow_patterns {
            if self.matches_address_pattern(addr, pattern) {
                return PolicyDecision::Allow;
            }
        }

        PolicyDecision::NoMatch
    }

    fn matches_address_pattern(&self, addr: &str, pattern: &str) -> bool {
        let parts: Vec<&str> = addr.split(':').collect();
        let pattern_parts: Vec<&str> = pattern.split(':').collect();

        if parts.len() != 2 || pattern_parts.len() != 2 {
            return false;
        }

        let (host, port) = (parts[0], parts[1]);
        let (pattern_host, pattern_port) = (pattern_parts[0], pattern_parts[1]);

        let host_matches = pattern_host == "*"
            || Pattern::new(pattern_host)
                .map(|p| p.matches(host))
                .unwrap_or(false);

        let port_matches = pattern_port == "*" || port == pattern_port;

        host_matches && port_matches
    }

    fn check_command(
        &self,
        command: &str,
        allow_commands: &[String],
        deny_commands: &[String],
    ) -> PolicyDecision {
        let cmd_name = Path::new(command)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(command);

        for denied in deny_commands {
            if cmd_name == denied || command == denied {
                return PolicyDecision::Deny;
            }
        }

        for allowed in allow_commands {
            if cmd_name == allowed || command == allowed {
                return PolicyDecision::Allow;
            }
        }

        PolicyDecision::NoMatch
    }
}
