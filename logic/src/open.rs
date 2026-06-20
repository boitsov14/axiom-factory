use crate::syntax::{Formula, Formula::*, Term, Term::*};

impl Term {
    /// `Term` の指定された深さの `Bound` を `t` で代入する。
    fn open_at(&mut self, depth: usize, t: &Self) {
        match self {
            Bound(i) if *i == depth => {
                *self = t.clone();
            }
            Var(_) | Bound(_) => {}
            Fn(_, args) => {
                for u in args {
                    u.open_at(depth, t);
                }
            }
        }
    }
}

impl Formula {
    /// `Formula` の `Bound(0)` を `t` で代入する。
    pub fn open(&mut self, t: &Term) {
        self.open_at(0, t);
    }

    /// `Formula` の指定された深さの `Bound` を `t` で代入する。
    fn open_at(&mut self, depth: usize, t: &Term) {
        match self {
            False => {}
            Atom(_, args) => {
                for u in args {
                    u.open_at(depth, t);
                }
            }
            Eq(u, v) => {
                u.open_at(depth, t);
                v.open_at(depth, t);
            }
            Not(p) => p.open_at(depth, t),
            And(p, q) | Or(p, q) | To(p, q) | Iff(p, q) => {
                p.open_at(depth, t);
                q.open_at(depth, t);
            }
            All { body, .. } | Ex { body, .. } => {
                body.open_at(depth + 1, t);
            }
        }
    }
}
