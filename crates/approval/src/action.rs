use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ReadFile { path: String },
    WriteFile { path: String },
    AppendFile { path: String },
    DeleteFile { path: String },
    CreateDir { path: String },
    DeleteDir { path: String },
    CopyFile { src: String, dst: String },
    MoveFile { src: String, dst: String },
    ListDir { path: String },

    HttpRequest { method: String, url: String },

    TcpConnect { host: String, port: u16 },
    TcpListen { host: String, port: u16 },

    UdpBind { host: String, port: u16 },
    UdpSendTo { host: String, port: u16 },

    UnixConnect { path: String },
    UnixListen { path: String },

    Exec { command: String, args: Vec<String> },

    EnvGet { name: String },

    WebhookServe { host: String, port: u16 },
    WatchFiles { patterns: Vec<String> },
}

impl Action {
    pub fn category(&self) -> ActionCategory {
        match self {
            Action::ReadFile { .. } => ActionCategory::FileRead,
            Action::WriteFile { .. }
            | Action::AppendFile { .. }
            | Action::DeleteFile { .. }
            | Action::CreateDir { .. }
            | Action::DeleteDir { .. }
            | Action::CopyFile { .. }
            | Action::MoveFile { .. } => ActionCategory::FileWrite,
            Action::ListDir { .. } => ActionCategory::FileRead,
            Action::HttpRequest { .. } => ActionCategory::Http,
            Action::TcpConnect { .. } | Action::TcpListen { .. } => ActionCategory::Tcp,
            Action::UdpBind { .. } | Action::UdpSendTo { .. } => ActionCategory::Udp,
            Action::UnixConnect { .. } | Action::UnixListen { .. } => ActionCategory::Unix,
            Action::Exec { .. } => ActionCategory::Exec,
            Action::EnvGet { .. } => ActionCategory::Env,
            Action::WebhookServe { .. } => ActionCategory::Http,
            Action::WatchFiles { .. } => ActionCategory::FileRead,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self.category() {
            ActionCategory::FileRead => "📖",
            ActionCategory::FileWrite => "📝",
            ActionCategory::Http => "🌐",
            ActionCategory::Tcp => "🔌",
            ActionCategory::Udp => "📡",
            ActionCategory::Unix => "🔗",
            ActionCategory::Exec => "⚙️",
            ActionCategory::Env => "🔧",
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::ReadFile { path } => write!(f, "READ {}", path),
            Action::WriteFile { path } => write!(f, "WRITE {}", path),
            Action::AppendFile { path } => write!(f, "APPEND {}", path),
            Action::DeleteFile { path } => write!(f, "DELETE {}", path),
            Action::CreateDir { path } => write!(f, "MKDIR {}", path),
            Action::DeleteDir { path } => write!(f, "RMDIR {}", path),
            Action::CopyFile { src, dst } => write!(f, "COPY {} -> {}", src, dst),
            Action::MoveFile { src, dst } => write!(f, "MOVE {} -> {}", src, dst),
            Action::ListDir { path } => write!(f, "LIST {}", path),
            Action::HttpRequest { method, url } => write!(f, "HTTP {} {}", method, url),
            Action::TcpConnect { host, port } => write!(f, "TCP CONNECT {}:{}", host, port),
            Action::TcpListen { host, port } => write!(f, "TCP LISTEN {}:{}", host, port),
            Action::UdpBind { host, port } => write!(f, "UDP BIND {}:{}", host, port),
            Action::UdpSendTo { host, port } => write!(f, "UDP SEND {}:{}", host, port),
            Action::UnixConnect { path } => write!(f, "UNIX CONNECT {}", path),
            Action::UnixListen { path } => write!(f, "UNIX LISTEN {}", path),
            Action::Exec { command, args } => {
                if args.is_empty() {
                    write!(f, "EXEC {}", command)
                } else {
                    write!(f, "EXEC {} {}", command, args.join(" "))
                }
            }
            Action::EnvGet { name } => write!(f, "ENV {}", name),
            Action::WebhookServe { host, port } => write!(f, "WEBHOOK SERVE {}:{}", host, port),
            Action::WatchFiles { patterns } => write!(f, "WATCH {}", patterns.join(", ")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    FileRead,
    FileWrite,
    Http,
    Tcp,
    Udp,
    Unix,
    Exec,
    Env,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Allow,
    Deny,
    AllowAlways,
    DenyAlways,
}
