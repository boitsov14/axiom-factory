use std::collections::HashSet;

use crate::syntax::{Formula, Id, Term};
use crate::vars::fresh_var;

impl Term {
    /// `Term` の自由変数 `x` を `repl` で置き換える。
    pub fn subst(&self, x: &str, repl: &Term) -> Term {
        use Term::*;
        match self {
            Var(y) if y == x => repl.clone(),
            Var(_) => self.clone(),
            Fn(id, args) => Fn(
                id.clone(),
                args.iter().map(|arg| arg.subst(x, repl)).collect(),
            ),
        }
    }

    /// `Term` の変数名 `old` を `new` に置き換える。
    fn rename_var(&self, old: &str, new: &str) -> Term {
        use Term::*;
        match self {
            Var(x) if x == old => Var(new.to_string()),
            Var(_) => self.clone(),
            Fn(id, args) => Fn(
                id.clone(),
                args.iter().map(|arg| arg.rename_var(old, new)).collect(),
            ),
        }
    }
}

impl Formula {
    /// `Formula` の自由変数 `x` を `repl` で置き換える。
    pub fn subst(&self, x: &str, repl: &Term) -> Formula {
        use Formula::*;
        match self {
            False => False,
            Atom(id, args) => Atom(
                id.clone(),
                args.iter().map(|arg| arg.subst(x, repl)).collect(),
            ),
            Eq(s, t) => Eq(s.subst(x, repl), t.subst(x, repl)),
            Not(p) => Not(Box::new(p.subst(x, repl))),
            And(p, q) => And(Box::new(p.subst(x, repl)), Box::new(q.subst(x, repl))),
            Or(p, q) => Or(Box::new(p.subst(x, repl)), Box::new(q.subst(x, repl))),
            To(p, q) => To(Box::new(p.subst(x, repl)), Box::new(q.subst(x, repl))),
            Iff(p, q) => Iff(Box::new(p.subst(x, repl)), Box::new(q.subst(x, repl))),
            All { v, sort, body } if v == x => All {
                v: v.clone(),
                sort: sort.clone(),
                body: body.clone(),
            },
            Ex { v, sort, body } if v == x => Ex {
                v: v.clone(),
                sort: sort.clone(),
                body: body.clone(),
            },
            All { v, sort, body } => {
                let repl_vars = repl.vars();
                if repl_vars.contains(v) {
                    let fresh = fresh_for(v, body, repl, x);
                    let renamed = body.rename_var(v, &fresh);
                    All {
                        v: fresh,
                        sort: sort.clone(),
                        body: Box::new(renamed.subst(x, repl)),
                    }
                } else {
                    All {
                        v: v.clone(),
                        sort: sort.clone(),
                        body: Box::new(body.subst(x, repl)),
                    }
                }
            }
            Ex { v, sort, body } => {
                let repl_vars = repl.vars();
                if repl_vars.contains(v) {
                    let fresh = fresh_for(v, body, repl, x);
                    let renamed = body.rename_var(v, &fresh);
                    Ex {
                        v: fresh,
                        sort: sort.clone(),
                        body: Box::new(renamed.subst(x, repl)),
                    }
                } else {
                    Ex {
                        v: v.clone(),
                        sort: sort.clone(),
                        body: Box::new(body.subst(x, repl)),
                    }
                }
            }
        }
    }

    /// `Formula` の変数名 `old` を `new` に置き換える。
    fn rename_var(&self, old: &str, new: &str) -> Formula {
        use Formula::*;
        match self {
            False => False,
            Atom(id, args) => Atom(
                id.clone(),
                args.iter().map(|arg| arg.rename_var(old, new)).collect(),
            ),
            Eq(s, t) => Eq(s.rename_var(old, new), t.rename_var(old, new)),
            Not(p) => Not(Box::new(p.rename_var(old, new))),
            And(p, q) => And(Box::new(p.rename_var(old, new)), Box::new(q.rename_var(old, new))),
            Or(p, q) => Or(Box::new(p.rename_var(old, new)), Box::new(q.rename_var(old, new))),
            To(p, q) => To(Box::new(p.rename_var(old, new)), Box::new(q.rename_var(old, new))),
            Iff(p, q) => Iff(Box::new(p.rename_var(old, new)), Box::new(q.rename_var(old, new))),
            All { v, .. } if v == old => self.clone(),
            Ex { v, .. } if v == old => self.clone(),
            All { v, sort, body } => All {
                v: v.clone(),
                sort: sort.clone(),
                body: Box::new(body.rename_var(old, new)),
            },
            Ex { v, sort, body } => Ex {
                v: v.clone(),
                sort: sort.clone(),
                body: Box::new(body.rename_var(old, new)),
            },
        }
    }
}

/// 捕獲回避に使う新しい変数名を作る。
fn fresh_for(base: &str, body: &Formula, repl: &Term, x: &str) -> Id {
    let mut avoid: HashSet<Id> = body.vars();
    avoid.extend(repl.vars());
    avoid.insert(x.to_string());
    fresh_var(base, &avoid)
}
