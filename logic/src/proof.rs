use crate::syntax::Formula;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Goal {
    pub hypotheses: Vec<Formula>,
    pub target: Formula,
}
