use std::fmt;

pub type Id = String;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Sort {
    Obj,
    Nat,
    Int,
    Rat,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Term {
    Var(Id),
    Bound(usize),
    Fn(Id, Vec<Term>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Formula {
    False,
    Atom(Id, Vec<Term>),
    Eq(Term, Term),
    Not(Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    To(Box<Formula>, Box<Formula>),
    Iff(Box<Formula>, Box<Formula>),
    All { v: Id, sort: Sort, body: Box<Formula> },
    Ex { v: Id, sort: Sort, body: Box<Formula> },
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Sort::*;
        match self {
            Obj => write!(f, "Obj"),
            Nat => write!(f, "Nat"),
            Int => write!(f, "Int"),
            Rat => write!(f, "Rat"),
        }
    }
}
