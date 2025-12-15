use crate::action::{Action, ApprovalDecision};
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Select};
use std::io::IsTerminal;

pub struct InteractiveApprover {
    term: Term,
}

impl InteractiveApprover {
    pub fn new() -> Self {
        Self {
            term: Term::stderr(),
        }
    }

    pub fn is_interactive(&self) -> bool {
        std::io::stdin().is_terminal()
    }

    pub fn prompt_action(&self, action: &Action) -> anyhow::Result<ApprovalDecision> {
        self.term.write_line("")?;
        self.term.write_line(&format!(
            "{} Action requires approval:",
            style("⚠️").yellow()
        ))?;
        self.term.write_line("")?;
        self.term
            .write_line(&format!("   {} {}", action.icon(), style(action).cyan()))?;
        self.term.write_line("")?;

        let options = &[
            "[y] Allow once",
            "[n] Deny",
            "[a] Allow always (add to session)",
            "[d] Deny always (add to session)",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(options)
            .default(0)
            .interact_on(&self.term)
            .map_err(|e| anyhow::anyhow!("Selection error: {}", e))?;

        Ok(match selection {
            0 => ApprovalDecision::Allow,
            1 => ApprovalDecision::Deny,
            2 => ApprovalDecision::AllowAlways,
            3 => ApprovalDecision::DenyAlways,
            _ => ApprovalDecision::Deny,
        })
    }

    pub fn prompt_preflight(&self, actions: &[Action]) -> anyhow::Result<PreflightDecision> {
        self.term.write_line("")?;
        self.term
            .write_line(&format!("{}", style("=== Pre-flight Analysis ===").bold()))?;
        self.term
            .write_line("The script will perform the following operations:")?;
        self.term.write_line("")?;

        let mut fs_actions = Vec::new();
        let mut net_actions = Vec::new();
        let mut exec_actions = Vec::new();

        for action in actions {
            match action {
                Action::ReadFile { .. }
                | Action::WriteFile { .. }
                | Action::AppendFile { .. }
                | Action::DeleteFile { .. }
                | Action::CreateDir { .. }
                | Action::DeleteDir { .. }
                | Action::CopyFile { .. }
                | Action::MoveFile { .. }
                | Action::ListDir { .. } => fs_actions.push(action),

                Action::HttpRequest { .. }
                | Action::TcpConnect { .. }
                | Action::TcpListen { .. }
                | Action::UdpBind { .. }
                | Action::UdpSendTo { .. }
                | Action::UnixConnect { .. }
                | Action::UnixListen { .. }
                | Action::WebhookServe { .. } => net_actions.push(action),

                Action::Exec { .. } => exec_actions.push(action),

                Action::EnvGet { .. } | Action::WatchFiles { .. } => {}
            }
        }

        if !fs_actions.is_empty() {
            self.term
                .write_line(&format!("{}", style("Filesystem:").underlined()))?;
            for action in &fs_actions {
                self.term
                    .write_line(&format!("  {} {}", action.icon(), action))?;
            }
            self.term.write_line("")?;
        }

        if !net_actions.is_empty() {
            self.term
                .write_line(&format!("{}", style("Network:").underlined()))?;
            for action in &net_actions {
                self.term
                    .write_line(&format!("  {} {}", action.icon(), action))?;
            }
            self.term.write_line("")?;
        }

        if !exec_actions.is_empty() {
            self.term
                .write_line(&format!("{}", style("Exec:").underlined()))?;
            for action in &exec_actions {
                self.term
                    .write_line(&format!("  {} {}", action.icon(), action))?;
            }
            self.term.write_line("")?;
        }

        let options = &[
            "[A] Approve all",
            "[D] Deny all",
            "[R] Review each",
            "[C] Cancel",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(options)
            .default(0)
            .interact_on(&self.term)
            .map_err(|e| anyhow::anyhow!("Selection error: {}", e))?;

        Ok(match selection {
            0 => PreflightDecision::ApproveAll,
            1 => PreflightDecision::DenyAll,
            2 => PreflightDecision::ReviewEach,
            3 => PreflightDecision::Cancel,
            _ => PreflightDecision::Cancel,
        })
    }
}

impl Default for InteractiveApprover {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightDecision {
    ApproveAll,
    DenyAll,
    ReviewEach,
    Cancel,
}
