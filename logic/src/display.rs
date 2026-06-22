use crate::syntax::{Formula, Id, Sort, Sort::*, Term};
use maplit::hashset;
use std::{collections::HashSet, fmt};

impl Term {
    /// `Term` を LaTeX 文字列に変換する。
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

    /// `Term` に出現する ID を集める。
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

/// 連続する同じ種類の束縛子（∀ または ∃）を収集し、本体への参照を返す。
fn collect_binders(formula: &Formula) -> (Vec<(bool, Vec<(Id, Sort)>)>, &Formula) {
    let mut groups: Vec<(bool, Vec<(Id, Sort)>)> = Vec::new();
    let mut current = formula;
    loop {
        match current {
            Formula::All { v, sort, body } => {
                match groups.last_mut() {
                    Some(last) if last.0 => last.1.push((v.clone(), sort.clone())),
                    _ => groups.push((true, vec![(v.clone(), sort.clone())])),
                }
                current = body;
            }
            Formula::Ex { v, sort, body } => {
                match groups.last_mut() {
                    Some(last) if !last.0 => last.1.push((v.clone(), sort.clone())),
                    _ => groups.push((false, vec![(v.clone(), sort.clone())])),
                }
                current = body;
            }
            _ => break,
        }
    }
    (groups, current)
}

impl Formula {
    /// `Formula` を LaTeX 文字列に変換する。
    fn to_text(&self, stack: &mut Vec<Id>, used: &mut HashSet<Id>) -> String {
        use Formula::*;
        match self {
            All { .. } | Ex { .. } => {
                let (groups, body) = collect_binders(self);
                self.binders_to_text(&groups, body, stack, used)
            }
            // ...rest of the match
            _ => self.base_to_text(stack, used),
        }
    }

    /// 収集済みの束縛子グループを使って整形する。
    fn binders_to_text(
        &self,
        groups: &[(bool, Vec<(Id, Sort)>)],
        body: &Formula,
        stack: &mut Vec<Id>,
        used: &mut HashSet<Id>,
    ) -> String {
        // 各グループの変数に fresh な名前を付け、スタックに積む
        let mut all_names: Vec<Vec<Id>> = Vec::new();
        for (_, vars) in groups {
            let mut names = Vec::new();
            for (v, _) in vars {
                let n = fresh(v, used);
                names.push(n.clone());
                used.insert(n.clone());
                stack.push(n);
            }
            all_names.push(names);
        }

        let body_str = body.to_text(stack, used);

        // スタックを戻す
        for names in all_names.iter().rev() {
            for _ in names {
                stack.pop();
            }
        }

        // 文字列を構築
        let mut result = String::new();
        for ((is_all, vars), names) in groups.iter().zip(all_names.iter()) {
            let quant = if *is_all { r"\forall " } else { r"\exists " };
            result.push_str(quant);

            let all_obj = vars.iter().all(|(_, s)| *s == Obj);
            let all_same_sort = vars.windows(2).all(|w| w[0].1 == w[1].1);

            if all_obj {
                result.push_str(&names.join(" "));
            } else if all_same_sort {
                let sort_str = format!("{}", vars[0].1);
                result.push_str(&format!("({} : {sort_str})", names.join(" ")));
            } else {
                let parts: Vec<String> = vars
                    .iter()
                    .zip(names.iter())
                    .map(|((_, s), n)| {
                        if *s == Obj {
                            n.clone()
                        } else {
                            let sort_str = format!("{s}");
                            format!("({n} : {sort_str})")
                        }
                    })
                    .collect();
                result.push_str(&parts.join(" "));
            }
            result.push_str(", ");
        }
        result.push_str(&body_str);
        result
    }

    fn base_to_text(&self, stack: &mut Vec<Id>, used: &mut HashSet<Id>) -> String {
        use Formula::*;
        match self {
            False => r"\bot".into(),
            Atom(pred, args) if args.is_empty() => pred.clone(),
            Atom(pred, args) => {
                let args = args
                    .iter()
                    .map(|t| t.to_text(stack))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{pred} {args}")
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
            _ => unreachable!(),
        }
    }

    /// `Formula` に出現する ID を集める。
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
        assert_eq!(parse_formula("x = y").unwrap().to_string(), r"(x = y)");
    }

    #[test]
    fn test_display_predicate_no_args() {
        assert_eq!(parse_formula("P").unwrap().to_string(), "P");
    }

    #[test]
    fn test_display_predicate_with_args() {
        assert_eq!(parse_formula("P x y").unwrap().to_string(), "P x y");
    }

    #[test]
    fn test_display_negation() {
        assert_eq!(parse_formula("¬ P").unwrap().to_string(), r"\lnot P");
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
        let f = parse_formula("∀ x, P x").unwrap();
        assert_eq!(f.to_string(), r"\forall x, P x");
    }

    #[test]
    fn test_display_forall_multi_vars() {
        let f = parse_formula("∀ x y z, P x y z").unwrap();
        assert_eq!(f.to_string(), r"\forall x y z, P x y z");
    }

    #[test]
    fn test_display_forall_typed_single() {
        let f = parse_formula("∀ (x : N), P x").unwrap();
        assert_eq!(f.to_string(), r"\forall (x : \mathbb{N}), P x");
    }

    #[test]
    fn test_display_forall_typed_group() {
        let f = parse_formula("∀ (x y z : N), P x y z").unwrap();
        assert_eq!(f.to_string(), r"\forall (x y z : \mathbb{N}), P x y z");
    }

    #[test]
    fn test_display_forall_multi_typed_groups() {
        let f = parse_formula("∀ (x : N) (y : Nat), P x y").unwrap();
        // Both Nat, same sort → one group
        assert_eq!(f.to_string(), r"\forall (x y : \mathbb{N}), P x y");
    }

    #[test]
    fn test_display_forall_mixed_sorts() {
        let f = parse_formula("∀ (x : N) (y : Obj), P x y").unwrap();
        // Different sorts: x has sort N, y is Obj
        assert_eq!(f.to_string(), r"\forall (x : \mathbb{N}) y, P x y");
    }

    #[test]
    fn test_display_exists_single() {
        let f = parse_formula("∃ x, P x").unwrap();
        assert_eq!(f.to_string(), r"\exists x, P x");
    }

    #[test]
    fn test_display_exists_multi_vars() {
        let f = parse_formula("∃ x y z, P x y z").unwrap();
        assert_eq!(f.to_string(), r"\exists x y z, P x y z");
    }

    #[test]
    fn test_display_forall_nested() {
        let f = parse_formula("∀ x, ∀ y, P x y").unwrap();
        // Nested ∀ with same sort → merged into one group
        assert_eq!(f.to_string(), r"\forall x y, P x y");
    }

    #[test]
    fn test_display_mixed_quantifier_alternation() {
        let f = parse_formula("∀ x, ∃ y, P x y").unwrap();
        assert_eq!(f.to_string(), r"\forall x, \exists y, P x y");
    }

    #[test]
    fn test_display_complex_mixed() {
        let f = parse_formula("∀ x y, ∃ z, P x y z").unwrap();
        assert_eq!(f.to_string(), r"\forall x y, \exists z, P x y z");
    }

    /// ∀ は最も優先順位が低いため ∀ x, (P x → Q) の形で表示される
    #[test]
    fn test_display_forall_scopes_over_to() {
        let f = parse_formula("∀ x, P x → Q").unwrap();
        assert_eq!(f.to_string(), r"\forall x, (P x \to Q)");
    }
}
