use crate::syntax::{Formula, Id, Sort};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hyp {
    pub name: Id,
    pub fml: Formula,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ctx {
    pub vars: Vec<(Id, Sort)>,
    pub hyps: Vec<Hyp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Goal {
    pub ctx: Ctx,
    pub target: Formula,
}

impl Ctx {
    /// 空の `Ctx` を作る。
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            hyps: Vec::new(),
        }
    }
}

impl Default for Ctx {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal {
    /// 空の `Ctx` と指定されたゴールを持つ `Goal` を作る。
    pub fn new(target: Formula) -> Self {
        Self {
            ctx: Ctx::new(),
            target,
        }
    }
}
