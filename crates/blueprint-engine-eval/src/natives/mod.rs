mod approval;
mod builtins;
mod console;
mod crypto;
mod file;
mod http;
mod json;
mod jwt;
mod parallel;
mod process;
mod random;
mod redact;
mod regex;
mod socket;
mod task;
mod time;
pub mod triggers;
mod websocket;

use crate::eval::Evaluator;

pub fn register_all(evaluator: &mut Evaluator) {
    approval::register(evaluator);
    builtins::register(evaluator);
    console::register(evaluator);
    crypto::register(evaluator);
    file::register(evaluator);
    http::register(evaluator);
    json::register(evaluator);
    jwt::register(evaluator);
    parallel::register(evaluator);
    process::register(evaluator);
    random::register(evaluator);
    redact::register(evaluator);
    regex::register(evaluator);
    socket::register(evaluator);
    task::register(evaluator);
    time::register(evaluator);
    triggers::register(evaluator);
    websocket::register(evaluator);
}
