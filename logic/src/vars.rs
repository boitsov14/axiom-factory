use std::collections::HashSet;

use crate::syntax::{Formula, Id, Term};

impl Term {
    /// `Term` の変数を集める。
    pub fn vars(&self) -> HashSet<Id> {
        let mut out = HashSet::new();
        self.vars_rec(&mut out);
        out
    }

    /// `Term` の変数を集める再帰処理。
    fn vars_rec(&self, out: &mut HashSet<Id>) {
        use Term::*;
        match self {
            Var(x) => {
                out.insert(x.clone());
            }
            Fn(_, args) => {
                for arg in args {
                    arg.vars_rec(out);
                }
            }
        }
    }

    /// 束縛変数を考慮して、`Term` の自由変数を集める。
    pub(crate) fn free_vars(&self, bound: &HashSet<Id>, out: &mut HashSet<Id>) {
        use Term::*;
        match self {
            Var(x) => {
                if !bound.contains(x) {
                    out.insert(x.clone());
                }
            }
            Fn(_, args) => {
                for arg in args {
                    arg.free_vars(bound, out);
                }
            }
        }
    }
}

impl Formula {
    /// `Formula` の自由変数を集める。
    pub fn free_vars(&self) -> HashSet<Id> {
        let mut bound = HashSet::new();
        let mut out = HashSet::new();
        self.free_vars_rec(&mut bound, &mut out);
        out
    }

    /// `Formula` の変数を集める。
    pub fn vars(&self) -> HashSet<Id> {
        let mut out = HashSet::new();
        self.vars_rec(&mut out);
        out
    }

    /// `Formula` の自由変数を集める再帰処理。
    fn free_vars_rec(&self, bound: &mut HashSet<Id>, out: &mut HashSet<Id>) {
        use Formula::*;
        match self {
            False => {}
            Atom(_, args) => {
                for arg in args {
                    arg.free_vars(bound, out);
                }
            }
            Eq(s, t) => {
                s.free_vars(bound, out);
                t.free_vars(bound, out);
            }
            Not(p) => p.free_vars_rec(bound, out),
            And(p, q) | Or(p, q) | To(p, q) | Iff(p, q) => {
                p.free_vars_rec(bound, out);
                q.free_vars_rec(bound, out);
            }
            All { v, body, .. } | Ex { v, body, .. } => {
                bound.insert(v.clone());
                body.free_vars_rec(bound, out);
                bound.remove(v);
            }
        }
    }

    /// `Formula` の変数を集める再帰処理。
    fn vars_rec(&self, out: &mut HashSet<Id>) {
        use Formula::*;
        match self {
            False => {}
            Atom(_, args) => {
                for arg in args {
                    out.extend(arg.vars());
                }
            }
            Eq(s, t) => {
                out.extend(s.vars());
                out.extend(t.vars());
            }
            Not(p) => p.vars_rec(out),
            And(p, q) | Or(p, q) | To(p, q) | Iff(p, q) => {
                p.vars_rec(out);
                q.vars_rec(out);
            }
            All { v, body, .. } | Ex { v, body, .. } => {
                out.insert(v.clone());
                body.vars_rec(out);
            }
        }
    }
}

/// `avoid` に含まれない新しい変数名を作る。
pub fn fresh_var(base: &str, avoid: &HashSet<Id>) -> Id {
    let mut x = base.to_string();
    while avoid.contains(&x) {
        x.push('\'');
    }
    x
}
