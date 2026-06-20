use crate::syntax::{Formula, Id, Sort};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Goal {
    pub hypotheses: Vec<Formula>,
    pub target: Formula,
}
