use crate::action::Action;

pub fn analyze_script(path: &std::path::Path) -> anyhow::Result<Vec<Action>> {
    let content = std::fs::read_to_string(path)?;

    let mut actions = Vec::new();
    extract_actions_from_source(&content, &mut actions);

    Ok(actions)
}

fn extract_actions_from_source(source: &str, actions: &mut Vec<Action>) {
    for line in source.lines() {
        let line = line.trim();

        if let Some(action) = parse_bp_call(line) {
            actions.push(action);
        }
    }
}

fn parse_bp_call(line: &str) -> Option<Action> {
    if line.starts_with('#') || line.is_empty() {
        return None;
    }

    if let Some(rest) = line.strip_prefix("__bp_read_file(") {
        if let Some(path) = extract_string_arg(rest) {
            return Some(Action::ReadFile { path });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_write_file(") {
        if let Some(path) = extract_first_string_arg(rest) {
            return Some(Action::WriteFile { path });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_append_file(") {
        if let Some(path) = extract_first_string_arg(rest) {
            return Some(Action::AppendFile { path });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_delete_file(") {
        if let Some(path) = extract_string_arg(rest) {
            return Some(Action::DeleteFile { path });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_mkdir(") {
        if let Some(path) = extract_string_arg(rest) {
            return Some(Action::CreateDir { path });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_mkdir_all(") {
        if let Some(path) = extract_string_arg(rest) {
            return Some(Action::CreateDir { path });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_rmdir(") {
        if let Some(path) = extract_string_arg(rest) {
            return Some(Action::DeleteDir { path });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_rmdir_all(") {
        if let Some(path) = extract_string_arg(rest) {
            return Some(Action::DeleteDir { path });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_list_dir(") {
        if let Some(path) = extract_string_arg(rest) {
            return Some(Action::ListDir { path });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_http_get(") {
        if let Some(url) = extract_first_string_arg(rest) {
            return Some(Action::HttpRequest {
                method: "GET".to_string(),
                url,
            });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_http_post(") {
        if let Some(url) = extract_first_string_arg(rest) {
            return Some(Action::HttpRequest {
                method: "POST".to_string(),
                url,
            });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_http_put(") {
        if let Some(url) = extract_first_string_arg(rest) {
            return Some(Action::HttpRequest {
                method: "PUT".to_string(),
                url,
            });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_http_delete(") {
        if let Some(url) = extract_first_string_arg(rest) {
            return Some(Action::HttpRequest {
                method: "DELETE".to_string(),
                url,
            });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_tcp_connect(") {
        if let Some((host, port)) = extract_host_port(rest) {
            return Some(Action::TcpConnect { host, port });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_tcp_listen(") {
        if let Some((host, port)) = extract_host_port(rest) {
            return Some(Action::TcpListen { host, port });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_udp_bind(") {
        if let Some((host, port)) = extract_host_port(rest) {
            return Some(Action::UdpBind { host, port });
        }
    }

    if let Some(rest) = line.strip_prefix("__bp_exec(") {
        if let Some(command) = extract_first_string_arg(rest) {
            return Some(Action::Exec {
                command,
                args: Vec::new(),
            });
        }
    }

    None
}

fn extract_string_arg(s: &str) -> Option<String> {
    let s = s.trim();
    if s.starts_with('"') {
        let end = s[1..].find('"')?;
        return Some(s[1..end + 1].to_string());
    }
    if s.starts_with('\'') {
        let end = s[1..].find('\'')?;
        return Some(s[1..end + 1].to_string());
    }
    None
}

fn extract_first_string_arg(s: &str) -> Option<String> {
    extract_string_arg(s)
}

fn extract_host_port(s: &str) -> Option<(String, u16)> {
    let s = s.trim();
    let host = extract_string_arg(s)?;

    let after_host = s.find(',')?;
    let port_part = s[after_host + 1..].trim();

    let port_end = port_part.find(|c: char| !c.is_ascii_digit()).unwrap_or(port_part.len());
    let port: u16 = port_part[..port_end].parse().ok()?;

    Some((host, port))
}
