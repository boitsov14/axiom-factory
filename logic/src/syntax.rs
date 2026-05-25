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

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Term::*;
        match self {
            Var(x) => write!(f, "{x}"),
            Fn(id, args) if args.is_empty() => write!(f, "{id}"),
            Fn(id, args) => {
                write!(f, "{id}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Formula::*;
        match self {
            False => write!(f, "False"),
            Atom(id, args) if args.is_empty() => write!(f, "{id}"),
            Atom(id, args) => {
                write!(f, "{id}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            Eq(s, t) => write!(f, "{s} = {t}"),
            Not(p) => write!(f, "not ({p})"),
            And(p, q) => write!(f, "({p}) and ({q})"),
            Or(p, q) => write!(f, "({p}) or ({q})"),
            To(p, q) => write!(f, "({p}) -> ({q})"),
            Iff(p, q) => write!(f, "({p}) <-> ({q})"),
            All { v, sort: Sort::Obj, body } => write!(f, "all {v}, {body}"),
            All { v, sort, body } => write!(f, "all {v}: {sort}, {body}"),
            Ex { v, sort: Sort::Obj, body } => write!(f, "ex {v}, {body}"),
            Ex { v, sort, body } => write!(f, "ex {v}: {sort}, {body}"),
        }
    }
}
