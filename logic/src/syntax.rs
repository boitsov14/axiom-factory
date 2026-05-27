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
    Fn(Id, Vec<Self>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Formula {
    False,
    Atom(Id, Vec<Term>),
    Eq(Term, Term),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    To(Box<Self>, Box<Self>),
    Iff(Box<Self>, Box<Self>),
    All { v: Id, sort: Sort, body: Box<Self> },
    Ex { v: Id, sort: Sort, body: Box<Self> },
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
