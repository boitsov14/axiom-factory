use crate::syntax::{Formula, Formula::*, Id, Sort, Sort::*, Term, Term::*};
use maplit::hashset;
use std::{collections::HashSet, fmt};

impl Term {
    /// `Term` を LaTeX 文字列に変換する。
    fn to_text(&self, stack: &[Id]) -> String {
        match self {
            Var(x) => x.clone(),
            Bound(i) => stack[stack.len() - 1 - *i].clone(),
            Fn(f, args) if args.is_empty() => f.clone(),
            Fn(f, args) => {
                let args = args
                    .iter()
                    .map(|t| t.to_text(stack))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{f}({args})")
            }
        }
    }

    /// `Term` に出現する ID を集める。
    fn ids(&self, out: &mut HashSet<Id>) {
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
    /// `Formula` を LaTeX 文字列に変換する。
    fn to_text(&self, stack: &mut Vec<Id>, used: &mut HashSet<Id>) -> String {
        match self {
            False => r"\bot".into(),
            Atom(pred, args) if args.is_empty() => pred.clone(),
            Atom(pred, args) => {
                let args = args
                    .iter()
                    .map(|t| t.to_text(stack))
                    .collect::<Vec<_>>()
                    .join(",");
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
                    Obj => format!(r"\forall {v} {body}"),
                    _ => format!(r"\forall {v}:{sort} {body}"),
                }
            }
            Ex { v, sort, body } => {
                let v = fresh(v, used);
                used.insert(v.clone());
                stack.push(v.clone());
                let body = body.to_text(stack, used);
                stack.pop();
                match sort {
                    Obj => format!(r"\exists {v} {body}"),
                    _ => format!(r"\exists {v}:{sort} {body}"),
                }
            }
        }
    }

    /// `Formula` に出現する ID を集める。
    fn ids(&self, used: &mut HashSet<Id>) {
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

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj => write!(f, r"\mathbb{{V}}"),
            Nat => write!(f, r"\mathbb{{N}}"),
            Int => write!(f, r"\mathbb{{Z}}"),
            Rat => write!(f, r"\mathbb{{Q}}"),
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
        let mut used = hashset!();
        self.ids(&mut used);
        write!(f, "{}", self.to_text(&mut vec![], &mut used))
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

#[cfg(test)]
mod tests {
    use crate::parser::parse_formula;

    // --- display (to_string) ---

    #[test]
    fn test_display_equality() {
        assert_eq!(parse_formula("x = y").unwrap().to_string(), "(x = y)");
    }

    #[test]
    fn test_display_predicate_no_args() {
        assert_eq!(parse_formula("P").unwrap().to_string(), "P");
    }

    #[test]
    fn test_display_predicate_with_args() {
        assert_eq!(parse_formula("P(x, y)").unwrap().to_string(), "P x y");
    }

    #[test]
    fn test_display_negation() {
        assert_eq!(parse_formula("¬P").unwrap().to_string(), r"\lnot P");
    }

    #[test]
    fn test_display_implication() {
        assert_eq!(parse_formula("P → Q").unwrap().to_string(), r"(P \to Q)");
    }

    #[test]
    fn test_display_conjunction() {
        assert_eq!(parse_formula("P ∧ Q").unwrap().to_string(), r"(P \land Q)");
    }

    #[test]
    fn test_display_disjunction() {
        assert_eq!(parse_formula("P ∨ Q").unwrap().to_string(), r"(P \lor Q)");
    }

    #[test]
    fn test_display_iff() {
        assert_eq!(
            parse_formula("P ↔ Q").unwrap().to_string(),
            r"(P \leftrightarrow Q)"
        );
    }

    // --- quantifier display ---

    #[test]
    fn test_display_forall_single() {
        let f = parse_formula("∀x P(x)").unwrap();
        assert_eq!(f.to_string(), r"\forall x, P x");
    }

    #[test]
    fn test_display_forall_multi_vars() {
        let f = parse_formula("∀x∀y∀z P(x, y, z)").unwrap();
        assert_eq!(f.to_string(), r"\forall x y z, P x y z");
    }

    #[test]
    fn test_display_forall_typed_single() {
        let f = parse_formula("∀x:N P(x)").unwrap();
        assert_eq!(f.to_string(), r"\forall (x : \mathbb{N}), P x");
    }

    #[test]
    fn test_display_forall_typed_group() {
        let f = parse_formula("∀x:N ∀y:N ∀z:N P(x, y, z)").unwrap();
        assert_eq!(f.to_string(), r"\forall (x y z : \mathbb{N}), P x y z");
    }

    #[test]
    fn test_display_forall_multi_typed_groups() {
        let f = parse_formula("∀x:N ∀y:Nat P(x, y)").unwrap();
        // Both Nat, same sort → one group
        assert_eq!(f.to_string(), r"\forall (x y : \mathbb{N}), P x y");
    }

    #[test]
    fn test_display_forall_mixed_sorts() {
        let f = parse_formula("∀x:N ∀y P(x, y)").unwrap();
        // Different sorts: x has sort N, y is Obj
        assert_eq!(f.to_string(), r"\forall (x : \mathbb{N}) y, P x y");
    }

    #[test]
    fn test_display_exists_single() {
        let f = parse_formula("∃x P(x)").unwrap();
        assert_eq!(f.to_string(), r"\exists x, P x");
    }

    #[test]
    fn test_display_exists_multi_vars() {
        let f = parse_formula("∃x∃y∃z P(x, y, z)").unwrap();
        assert_eq!(f.to_string(), r"\exists x y z, P x y z");
    }

    #[test]
    fn test_display_forall_nested() {
        let f = parse_formula("∀x∀y P(x, y)").unwrap();
        // Nested ∀ with same sort → merged into one group
        assert_eq!(f.to_string(), r"\forall x y, P x y");
    }

    #[test]
    fn test_display_mixed_quantifier_alternation() {
        let f = parse_formula("∀x∃y P(x, y)").unwrap();
        assert_eq!(f.to_string(), r"\forall x, \exists y, P x y");
    }

    #[test]
    fn test_display_complex_mixed() {
        let f = parse_formula("∀x∀y∃z P(x, y, z)").unwrap();
        assert_eq!(f.to_string(), r"\forall x y, \exists z, P x y z");
    }

    /// ∀ は ¬, ∀, ∃ > ∧ > ∨ > → > ↔ の優先順位で → より高い。
    /// 明示的な括弧により ∀ が → にスコープする。
    #[test]
    fn test_display_forall_scopes_over_to_with_parens() {
        let f = parse_formula("∀x (P(x) → Q)").unwrap();
        assert_eq!(f.to_string(), r"\forall x, (P x \to Q)");
    }
}
