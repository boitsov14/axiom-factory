use crate::syntax::{Formula, Id, Sort, Term};
use std::{collections::HashSet, fmt};

impl Term {
    /// `Term` を文字列に変換する。
    fn to_text(&self, stack: &[Id]) -> String {
        use Term::*;
        match self {
            Var(x) => x.clone(),
            Bound(i) => stack[stack.len() - 1 - *i].clone(),
            Fn(f, args) if args.is_empty() => f.clone(),
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

    /// `Term` の ID を再帰的に集める。
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
    /// `Formula` を文字列に変換する。
    fn to_text(&self, stack: &mut Vec<Id>, used: &mut HashSet<Id>) -> String {
        use Formula::*;
        match self {
            False => r"\bot".into(),
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
            Not(p) => format!(r"\lnot {}", p.to_text(stack, used)),
            And(p, q) => format!(
                r"({} \land {})",
                p.to_text(stack, used),
                q.to_text(stack, used)
            ),
            Or(p, q) => format!(
                r"({} \lor {})",
                p.to_text(stack, used),
                q.to_text(stack, used)
            ),
            To(p, q) => format!(
                r"({} \to {})",
                p.to_text(stack, used),
                q.to_text(stack, used)
            ),
            Iff(p, q) => format!(
                r"({} \leftrightarrow {})",
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
                    Sort::Obj => format!(r"\forall {v}, {body}"),
                    _ => format!(r"\forall {v} : {sort}, {body}"),
                }
            }
            Ex { v, sort, body } => {
                let v = fresh(v, used);
                used.insert(v.clone());
                stack.push(v.clone());
                let body = body.to_text(stack, used);
                stack.pop();
                match sort {
                    Sort::Obj => format!(r"\exists {v}, {body}"),
                    _ => format!(r"\exists {v} : {sort}, {body}"),
                }
            }
        }
    }

    /// `Formula` の ID を再帰的に集める。
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
    let mut x = base.to_owned();
    while avoid.contains(&x) {
        x.push('\'');
    }
    x
}
