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
        let s = self.to_text(&mut vec![], &mut used);
        // 一番外側の括弧を削除する
        let s = s
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&s);
        write!(f, "{s}")
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
    use crate::parser::{parse_formula, parse_term};

    // --- Term ---

    #[test]
    fn test_display_term_variable() {
        assert_eq!(parse_term("x").unwrap().to_string(), "x");
    }

    #[test]
    fn test_display_term_unary_function() {
        assert_eq!(parse_term("f(x)").unwrap().to_string(), "f(x)");
    }

    #[test]
    fn test_display_term_binary_function() {
        assert_eq!(parse_term("f(x,y)").unwrap().to_string(), "f(x,y)");
    }

    #[test]
    fn test_display_term_nested_function() {
        assert_eq!(parse_term("f(g(x))").unwrap().to_string(), "f(g(x))");
    }

    #[test]
    fn test_display_term_with_apostrophe() {
        assert_eq!(parse_term("x'").unwrap().to_string(), "x'");
        assert_eq!(parse_term("f(x')").unwrap().to_string(), "f(x')");
    }

    // --- False ---

    #[test]
    fn test_display_false_keyword() {
        assert_eq!(parse_formula("false").unwrap().to_string(), r"\bot");
    }

    // --- Equality ---

    #[test]
    fn test_display_equality() {
        assert_eq!(parse_formula("x = y").unwrap().to_string(), "x = y");
    }

    #[test]
    fn test_display_equality_with_functions() {
        assert_eq!(
            parse_formula("f(x) = g(y)").unwrap().to_string(),
            "f(x) = g(y)"
        );
    }

    #[test]
    fn test_display_equality_nested_term() {
        assert_eq!(
            parse_formula("f(g(x)) = h(y)").unwrap().to_string(),
            "f(g(x)) = h(y)"
        );
    }

    // --- Predicate ---

    #[test]
    fn test_display_predicate_no_args() {
        assert_eq!(parse_formula("P").unwrap().to_string(), "P");
    }

    #[test]
    fn test_display_predicate_with_args() {
        assert_eq!(parse_formula("P(x,y)").unwrap().to_string(), "P(x,y)");
    }

    #[test]
    fn test_display_predicate_function_arg() {
        assert_eq!(
            parse_formula("P(f(x),g(y))").unwrap().to_string(),
            "P(f(x),g(y))"
        );
    }

    // --- Connectives ---

    #[test]
    fn test_display_negation() {
        assert_eq!(parse_formula("¬P").unwrap().to_string(), r"\lnot P");
    }

    #[test]
    fn test_display_double_negation() {
        assert_eq!(parse_formula("¬¬P").unwrap().to_string(), r"\lnot \lnot P");
    }

    #[test]
    fn test_display_conjunction() {
        assert_eq!(parse_formula("P ∧ Q").unwrap().to_string(), r"P \land Q");
    }

    #[test]
    fn test_display_disjunction() {
        assert_eq!(parse_formula("P ∨ Q").unwrap().to_string(), r"P \lor Q");
    }

    #[test]
    fn test_display_implication() {
        assert_eq!(parse_formula("P → Q").unwrap().to_string(), r"P \to Q");
    }

    #[test]
    fn test_display_iff() {
        assert_eq!(
            parse_formula("P ↔ Q").unwrap().to_string(),
            r"P \leftrightarrow Q"
        );
    }

    // --- Quantifier: Forall ---

    #[test]
    fn test_display_forall_single() {
        let f = parse_formula("∀x P(x)").unwrap();
        assert_eq!(f.to_string(), r"\forall x P(x)");
    }

    #[test]
    fn test_display_forall_nested() {
        let f = parse_formula("∀x∀y P(x,y)").unwrap();
        assert_eq!(f.to_string(), r"\forall x \forall y P(x,y)");
    }

    #[test]
    fn test_display_forall_typed_nat() {
        let f = parse_formula("∀x:N P(x)").unwrap();
        assert_eq!(f.to_string(), r"\forall x:\mathbb{N} P(x)");
    }

    #[test]
    fn test_display_forall_typed_int_short() {
        let f = parse_formula("∀x:Z P(x)").unwrap();
        assert_eq!(f.to_string(), r"\forall x:\mathbb{Z} P(x)");
    }

    #[test]
    fn test_display_forall_typed_rat_short() {
        let f = parse_formula("∀x:Q P(x)").unwrap();
        assert_eq!(f.to_string(), r"\forall x:\mathbb{Q} P(x)");
    }

    #[test]
    fn test_display_forall_typed_obj() {
        let f = parse_formula("∀x:V P(x)").unwrap();
        // Obj のソート注釈は表示されない
        assert_eq!(f.to_string(), r"\forall x P(x)");
    }

    #[test]
    fn test_display_forall_typed_group() {
        let f = parse_formula("∀x:N ∀y:N ∀z:N P(x,y,z)").unwrap();
        assert_eq!(
            f.to_string(),
            r"\forall x:\mathbb{N} \forall y:\mathbb{N} \forall z:\mathbb{N} P(x,y,z)"
        );
    }

    #[test]
    fn test_display_forall_mixed_typed_groups() {
        let f = parse_formula("∀x:N ∀y:Z P(x,y)").unwrap();
        assert_eq!(
            f.to_string(),
            r"\forall x:\mathbb{N} \forall y:\mathbb{Z} P(x,y)"
        );
    }

    #[test]
    fn test_display_negation_over_forall() {
        // ¬∀x P(x) = ¬(∀x P(x))
        let f = parse_formula("¬∀x P(x)").unwrap();
        assert_eq!(f.to_string(), r"\lnot \forall x P(x)");
    }

    #[test]
    fn test_display_forall_scopes_over_to() {
        // ∀x P(x) → Q = (∀x P(x)) → Q
        let f = parse_formula("∀x P(x) → Q").unwrap();
        assert_eq!(f.to_string(), r"\forall x P(x) \to Q");
    }

    // --- Quantifier: Exists ---

    #[test]
    fn test_display_exists_single() {
        let f = parse_formula("∃x P(x)").unwrap();
        assert_eq!(f.to_string(), r"\exists x P(x)");
    }

    #[test]
    fn test_display_exists_multi_vars() {
        let f = parse_formula("∃x∃y∃z P(x,y,z)").unwrap();
        assert_eq!(f.to_string(), r"\exists x \exists y \exists z P(x,y,z)");
    }

    #[test]
    fn test_display_exists_typed_nat() {
        let f = parse_formula("∃x:N P(x)").unwrap();
        assert_eq!(f.to_string(), r"\exists x:\mathbb{N} P(x)");
    }

    #[test]
    fn test_display_exists_typed_int() {
        let f = parse_formula("∃x:Z P(x)").unwrap();
        assert_eq!(f.to_string(), r"\exists x:\mathbb{Z} P(x)");
    }

    #[test]
    fn test_display_exists_typed_rat() {
        let f = parse_formula("∃x:Q P(x)").unwrap();
        assert_eq!(f.to_string(), r"\exists x:\mathbb{Q} P(x)");
    }

    #[test]
    fn test_display_exists_typed_obj() {
        let f = parse_formula("∃x:V P(x)").unwrap();
        // Obj のソート注釈は表示されない
        assert_eq!(f.to_string(), r"\exists x P(x)");
    }

    #[test]
    fn test_display_exists_mixed_typed() {
        let f = parse_formula("∃x:N ∃y:Z P(x,y)").unwrap();
        assert_eq!(
            f.to_string(),
            r"\exists x:\mathbb{N} \exists y:\mathbb{Z} P(x,y)"
        );
    }

    // --- Mixed quantifiers ---

    #[test]
    fn test_display_mixed_quantifier_alternation() {
        let f = parse_formula("∀x∃y P(x,y)").unwrap();
        assert_eq!(f.to_string(), r"\forall x \exists y P(x,y)");
    }

    #[test]
    fn test_display_complex_mixed() {
        let f = parse_formula("∀x∀y∃z P(x,y,z)").unwrap();
        assert_eq!(f.to_string(), r"\forall x \forall y \exists z P(x,y,z)");
    }

    #[test]
    fn test_display_mixed_typed_quantifiers() {
        let f = parse_formula("∀x:N ∃y:Z P(x,y)").unwrap();
        assert_eq!(
            f.to_string(),
            r"\forall x:\mathbb{N} \exists y:\mathbb{Z} P(x,y)"
        );
    }

    // --- Fresh variable renaming ---

    #[test]
    fn test_display_fresh_renaming() {
        // ∀x P(x) ∧ ∀x Q(x) — 内側の x は x' にリネームされる
        let f = parse_formula("∀x P(x) ∧ ∀x Q(x)").unwrap();
        assert_eq!(f.to_string(), r"\forall x P(x) \land \forall x' Q(x')");
    }

    #[test]
    fn test_display_fresh_renaming_multi_apostrophe() {
        // P(x) の自由変数 x も used に含まれるため、
        // 1つ目の ∀x は x' に、2つ目の ∀x は x'' にリネームされる
        let f = parse_formula("P(x) ∧ ∀x Q(x) ∧ ∀x R(x)").unwrap();
        assert_eq!(
            f.to_string(),
            r"P(x) \land (\forall x' Q(x') \land \forall x'' R(x''))"
        );
    }

    #[test]
    fn test_display_free_variable_preserved() {
        // ∀x∀y P(x,y,z) — z は自由変数のまま
        let f = parse_formula("∀x∀y P(x,y,z)").unwrap();
        assert_eq!(f.to_string(), r"\forall x \forall y P(x,y,z)");
    }
}
