use crate::syntax::{Formula, Term};

impl Term {
    /// `Term` の指定された深さの `Bound` を `Var(x)` に開く。
    fn open_rec(&mut self, depth: usize, x: &str) {
        use Term::*;
        match self {
            Bound(i) if *i == depth => {
                *self = Var(x.to_string());
            }
            Var(_) | Bound(_) => {}
            Fn(_, args) => {
                for t in args {
                    t.open_rec(depth, x);
                }
            }
        }
    }
}

impl Formula {
    /// `Formula` の `Bound(0)` を `Var(x)` に開く。
    pub fn open(&mut self, x: &str) {
        self.open_rec(0, x);
    }

    /// `Formula` の指定された深さの `Bound` を `Var(x)` に開く。
    fn open_rec(&mut self, depth: usize, x: &str) {
        use Formula::*;
        match self {
            False => {}

            Atom(_, args) => {
                for t in args {
                    t.open_rec(depth, x);
                }
            }

            Eq(s, t) => {
                s.open_rec(depth, x);
                t.open_rec(depth, x);
            }

            Not(p) => p.open_rec(depth, x),

            And(p, q) | Or(p, q) | To(p, q) | Iff(p, q) => {
                p.open_rec(depth, x);
                q.open_rec(depth, x);
            }

            All { body, .. } | Ex { body, .. } => {
                body.open_rec(depth + 1, x);
            }
        }
    }
}
