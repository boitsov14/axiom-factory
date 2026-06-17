use crate::syntax::{Formula, Id, Sort};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hyp {
    pub id: Id,
    pub formula: Formula,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Context {
    pub vars: Vec<(Id, Sort)>,
    pub hyps: Vec<Hyp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Goal {
    pub ctx: Context,
    pub target: Formula,
}

impl Goal {
    /// 空の `Context` を持つ `Goal` を作る。
    pub fn new(target: Formula) -> Self {
        Self {
            ctx: Context::default(),
            target,
        }
    }
}
