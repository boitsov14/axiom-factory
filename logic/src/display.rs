use crate::syntax::{Formula, Id, Sort, Term};
use std::collections::HashSet;
use std::fmt;

impl Term {
    /// `Term` を binder stack に従って表示用テキストに変換する。
    fn to_text(&self, stack: &[Id]) -> String {
        use Term::*;
        match self {
            Var(x) => x.clone(),
            Bound(i) => stack
                .len()
                .checked_sub(i + 1)
                .and_then(|j| stack.get(j))
                .cloned()
                .unwrap_or_else(|| format!("#{i}")),
            Fn(f, args) if args.is_empty() => format!("{f}()"),
            Fn(f, args) => {
                let args = args
                    .iter()
                    .map(|t| t.to_text(stack))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{f}({args})")
            }
        }
    }

    /// `Term` の表示衝突に関わる ID を再帰的に集める。
    fn ids(&self, out: &mut HashSet<Id>) {
        use Term::*;
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
    /// `Formula` を binder stack と使用済み ID に従って表示用テキストに変換する。
    fn to_text(&self, stack: &mut Vec<Id>, used: &mut HashSet<Id>) -> String {
        use Formula::*;
        match self {
            False => "False".to_string(),
            Atom(pred, args) if args.is_empty() => pred.clone(),
            Atom(pred, args) => {
                let args = args
                    .iter()
                    .map(|t| t.to_text(stack))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{pred}({args})")
            }
            Eq(s, t) => format!("({} = {})", s.to_text(stack), t.to_text(stack)),
            Not(p) => format!("not {}", p.to_text(stack, used)),
            And(p, q) => format!(
                "({} and {})",
                p.to_text(stack, used),
                q.to_text(stack, used)
            ),
            Or(p, q) => format!("({} or {})", p.to_text(stack, used), q.to_text(stack, used)),
            To(p, q) => format!("({} -> {})", p.to_text(stack, used), q.to_text(stack, used)),
            Iff(p, q) => format!(
                "({} <-> {})",
                p.to_text(stack, used),
                q.to_text(stack, used)
            ),
            All { v, sort, body } => {
                let v = fresh(v, used);
                used.insert(v.clone());
                stack.push(v.clone());
                let body = body.to_text(stack, used);
                stack.pop();
                match sort {
                    Sort::Obj => format!("all {v}, {body}"),
                    _ => format!("all {v} : {sort}, {body}"),
                }
            }
            Ex { v, sort, body } => {
                let v = fresh(v, used);
                used.insert(v.clone());
                stack.push(v.clone());
                let body = body.to_text(stack, used);
                stack.pop();
                match sort {
                    Sort::Obj => format!("ex {v}, {body}"),
                    _ => format!("ex {v} : {sort}, {body}"),
                }
            }
        }
    }

    /// `Formula` の表示衝突に関わる ID を再帰的に集める。
    fn ids(&self, used: &mut HashSet<Id>) {
        use Formula::*;
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

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text(&[]))
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut used = HashSet::new();
        self.ids(&mut used);
        write!(f, "{}", self.to_text(&mut Vec::new(), &mut used))
    }
}

/// `avoid` に含まれない ID を作る。
pub fn fresh(base: &str, avoid: &HashSet<Id>) -> Id {
    let mut x = base.to_string();
    while avoid.contains(&x) {
        x.push('\'');
    }
    x
}
