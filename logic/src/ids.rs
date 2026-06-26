use crate::syntax::{Formula, Formula::*, Id, Term, Term::*};
use std::collections::HashSet;

impl Term {
    /// `Term` に出現する ID を集める。
    pub(crate) fn ids(&self, out: &mut HashSet<Id>) {
        match self {
            Var(x) => {
                out.insert(x.clone());
            }
            Bound(_) => {}
            Fn(f, args) => {
                out.insert(f.clone());
                for t in args {
                    t.ids(out);
                }
            }
        }
    }
}

impl Formula {
    /// `Formula` に出現する ID を集める。
    pub(crate) fn ids(&self, used: &mut HashSet<Id>) {
        match self {
            False => {}
            Atom(pred, args) => {
                used.insert(pred.clone());
                for t in args {
                    t.ids(used);
                }
            }
            Eq(s, t) => {
                s.ids(used);
                t.ids(used);
            }
            Not(p) => p.ids(used),
            And(p, q) | Or(p, q) | To(p, q) | Iff(p, q) => {
                p.ids(used);
                q.ids(used);
            }
            All { body, .. } | Ex { body, .. } => body.ids(used),
        }
    }
}

/// `avoid` に含まれない ID を作る。
pub fn fresh(base: &str, avoid: &HashSet<Id>) -> Id {
    let mut x = base.to_owned();
    while avoid.contains(&x) {
        x.push('\'');
    }
    x
}
