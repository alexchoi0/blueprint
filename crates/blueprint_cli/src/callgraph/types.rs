use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Entry,
    Exit,
    Statement,
    Condition,
    ForLoop,
    Match,
    Yield,
    Import,
    Export,
}

#[derive(Debug, Clone)]
pub struct CfgNode {
    pub id: usize,
    pub kind: NodeKind,
    pub label: String,
    pub file: PathBuf,
    pub function: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Sequential,
    TrueBranch,
    FalseBranch,
    LoopBack,
    LoopDone,
    LoopBreak,
    Call,
    Imports,
    Exports,
}

#[derive(Debug, Clone)]
pub struct CfgEdge {
    pub from: usize,
    pub to: usize,
    pub kind: EdgeKind,
}
