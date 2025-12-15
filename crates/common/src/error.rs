use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlueprintError {
    #[error("Action denied by policy: {action}")]
    PolicyDenied { action: String },

    #[error("Action denied by user: {action}")]
    UserDenied { action: String },

    #[error("Non-interactive mode: action requires approval: {action}")]
    NonInteractiveDenied { action: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type BlueprintResult<T> = Result<T, BlueprintError>;
