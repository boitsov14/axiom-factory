use crate::{
    parser::parser::{formula, sequent, term},
    syntax::{Formula, Id, Term},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Found {lp} left parentheses and {rp} right parentheses.")]
    Parentheses { lp: usize, rp: usize },
    #[error("parse error at line {line}, column {column}: expected {expected}")]
    Peg {
        line: usize,
        column: usize,
        expected: String,
    },
    #[error("sequent must have exactly one target formula")]
    TargetCount,
}

/// 項を構文解析する。
///
/// # Errors
///
/// 入力が項として構文解析できない場合、または括弧数が一致しない場合にエラーを返す。
pub fn parse_term(s: &str) -> Result<Term, Error> {
    let s = clean(s);
    check_parentheses(&s)?;
    term(&s).map_err(|e| Error::Peg {
        line: e.location.line,
        column: e.location.column,
        expected: e.expected.to_string(),
    })
}

/// 論理式を構文解析する。
///
/// # Errors
///
/// 入力が論理式として構文解析できない場合、または括弧数が一致しない場合にエラーを返す。
pub fn parse_formula(s: &str) -> Result<Formula, Error> {
    let s = clean(s);
    check_parentheses(&s)?;
    let mut formula = formula(&s).map_err(|e| Error::Peg {
        line: e.location.line,
        column: e.location.column,
        expected: e.expected.to_string(),
    })?;
    close_formula(&mut formula);
    Ok(formula)
}

/// シーケントを `Goal` 相当の組に構文解析する。
///
/// # Errors
///
/// 入力がシーケントとして構文解析できない場合、括弧数が一致しない場合、または結論が複数ある場合にエラーを返す。
pub fn parse_goal(s: &str) -> Result<(Vec<Formula>, Formula), Error> {
    let s = clean(s);
    check_parentheses(&s)?;
    let (mut hypotheses, targets) = sequent(&s).map_err(|e| Error::Peg {
        line: e.location.line,
        column: e.location.column,
        expected: e.expected.to_string(),
    })?;

    for p in &mut hypotheses {
        close_formula(p);
    }

    let mut target = match targets.len() {
        0 => Formula::False,
        1 => targets.into_iter().next().ok_or(Error::TargetCount)?,
        _ => return Err(Error::TargetCount),
    };
    close_formula(&mut target);

    Ok((hypotheses, target))
}

/// 入力文字列の空白を整理する。
fn clean(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 括弧の数を確認する。
fn check_parentheses(s: &str) -> Result<(), Error> {
    let lp = s.chars().filter(|&c| c == '(').count();
    let rp = s.chars().filter(|&c| c == ')').count();
    if lp == rp {
        Ok(())
    } else {
        Err(Error::Parentheses { lp, rp })
    }
}

/// 束縛変数名を de Bruijn index に変換する。
fn close_formula(p: &mut Formula) {
    close_formula_at(p, &mut vec![]);
}

/// 現在の束縛変数スタックに従って `Formula` を閉じる。
fn close_formula_at(p: &mut Formula, stack: &mut Vec<Id>) {
    match p {
        Formula::False => {}
        Formula::Atom(_, args) => {
            for t in args {
                close_term_at(t, stack);
            }
        }
        Formula::Eq(s, t) => {
            close_term_at(s, stack);
            close_term_at(t, stack);
        }
        Formula::Not(q) => close_formula_at(q, stack),
        Formula::And(q, r) | Formula::Or(q, r) | Formula::To(q, r) | Formula::Iff(q, r) => {
            close_formula_at(q, stack);
            close_formula_at(r, stack);
        }
        Formula::All { v, body, .. } | Formula::Ex { v, body, .. } => {
            stack.push(v.clone());
            close_formula_at(body, stack);
            stack.pop();
        }
    }
}

/// 現在の束縛変数スタックに従って `Term` を閉じる。
fn close_term_at(t: &mut Term, stack: &[Id]) {
    match t {
        Term::Var(x) => {
            if let Some(i) = stack.iter().rev().position(|v| v == x) {
                *t = Term::Bound(i);
            }
        }
        Term::Bound(_) => {}
        Term::Fn(_, args) => {
            for u in args {
                close_term_at(u, stack);
            }
        }
    }
}

peg::parser!(grammar parser() for str {
    use crate::syntax::{Formula, Id, Sort, Term};

    pub rule term() -> Term = quiet!{
        f:$func_id() _ "(" _ ts:(term() ++ (_ "," _)) _ ")" { Term::Fn(f.into(), ts) } /
        v:$var_id() { Term::Var(v.into()) } /
        "(" _ t:term() _ ")" { t }
    } / expected!("term")

    rule atom() -> Formula =
        p_true() { Formula::To(Box::new(Formula::False), Box::new(Formula::False)) } /
        p_false() { Formula::False } /
        s:term() _ eq() _ t:term() { Formula::Eq(s, t) } /
        p:$pred_id() ts:(_ t:term() {t})* { Formula::Atom(p.into(), ts) }

    pub rule formula() -> Formula = precedence!{
        all() _ vs:quant_vars() _ p:@ {
            vs.into_iter().rev().fold(p, |p, (v, sort)| Formula::All { v, sort, body: Box::new(p) })
        }
        ex() _ vs:quant_vars() _ p:@ {
            vs.into_iter().rev().fold(p, |p, (v, sort)| Formula::Ex { v, sort, body: Box::new(p) })
        }
        --
        p:@ _ iff() _ q:(@) { Formula::Iff(Box::new(p), Box::new(q)) }
        --
        p:@ _ to() _ q:(@) { Formula::To(Box::new(p), Box::new(q)) }
        --
        p:@ _ or() _ q:(@) { Formula::Or(Box::new(p), Box::new(q)) }
        --
        p:@ _ and() _ q:(@) { Formula::And(Box::new(p), Box::new(q)) }
        --
        not() _ p:@ { Formula::Not(Box::new(p)) }
        --
        p:atom() { p }
        "(" _ p:formula() _ ")" { p }
    } / expected!("formula")

    pub rule sequent() -> (Vec<Formula>, Vec<Formula>) =
        hypotheses:(formula() ** (_ "," _)) _ turnstile() _ targets:(formula() ** (_ "," _)) {
            (hypotheses, targets)
        } /
        p:formula() { (vec![], vec![p]) } /
        expected!("sequent")

    rule quant_vars() -> Vec<(Id, Sort)> =
        gs:(quant_group() ++ (_)) _ "," { gs.into_iter().flatten().collect() } /
        vs:($(bdd_var_id()) ++ (_)) _ "," { vs.into_iter().map(|v| (v.to_owned(), Sort::Obj)).collect() }

    rule quant_group() -> Vec<(Id, Sort)> =
        "(" _ vs:($(bdd_var_id()) ++ (_)) _ ":" _ s:sort() _ ")" {
            vs.into_iter().map(|v| (v.to_owned(), s.clone())).collect()
        }

    rule sort() -> Sort = quiet!{
        "Obj" { Sort::Obj } /
        "V" { Sort::Obj } /
        "Nat" { Sort::Nat } /
        "N" { Sort::Nat } /
        r"\mathbb{N}" { Sort::Nat } /
        "Int" { Sort::Int } /
        "Z" { Sort::Int } /
        r"\mathbb{Z}" { Sort::Int } /
        "Rat" { Sort::Rat } /
        "Q" { Sort::Rat } /
        r"\mathbb{Q}" { Sort::Rat }
    } / expected!("sort")

    rule alpha() = ['a'..='z' | 'A'..='Z']
    rule digit() = ['0'..='9' | '_' | '\'']
    rule id() = alpha() (alpha() / digit())*
    rule var_id() = id()
    rule bdd_var_id() = alpha() digit()*
    rule func_id() = id()
    rule pred_id() = quiet!{ id() } / expected!("predicate")
    rule p_true() = quiet!{ "⊤" / "true" / r"\top" }
    rule p_false() = quiet!{ "⊥" / "⟂" / "false" / r"\bot" }
    rule not() = quiet!{ "¬" / "~" / "not" / r"\lnot" / r"\neg" } / expected!("¬")
    rule and() = quiet!{ "∧" / r"/\" / "&" / "and" / r"\land" / r"\wedge" } / expected!("∧")
    rule or() = quiet!{ "∨" / r"\/" / "|" / "or" / r"\lor" / r"\vee" } / expected!("∨")
    rule to() = quiet!{ "→" / "->" / "=>" / "to" / r"\rightarrow" / r"\to" } / expected!("→")
    rule iff() = quiet!{ "↔" / "<->" / "<=>" / "iff" / r"\leftrightarrow" } / expected!("↔")
    rule all() = quiet!{ "∀" / "!" / "all" / r"\forall" } / expected!("∀")
    rule ex() = quiet!{ "∃" / "?" / "ex" / r"\exists" } / expected!("∃")
    rule eq() = quiet!{ "=" } / expected!("=")
    rule comma() = quiet!{ "," }
    rule turnstile() = quiet!{ "⊢" / "|-" / "├" / "┣" / r"\vdash" } / expected!("⊢")
    rule _ = quiet!{ [' ']* }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Sort;

    // --- parse_term ---

    #[test]
    fn test_parse_term_simple_variable() {
        assert_eq!(parse_term("x").unwrap(), Term::Var("x".into()));
    }

    #[test]
    fn test_parse_term_unary_function() {
        assert_eq!(
            parse_term("f(x)").unwrap(),
            Term::Fn("f".into(), vec![Term::Var("x".into())])
        );
    }

    #[test]
    fn test_parse_term_binary_function() {
        assert_eq!(
            parse_term("f(x, y)").unwrap(),
            Term::Fn(
                "f".into(),
                vec![Term::Var("x".into()), Term::Var("y".into())]
            )
        );
    }

    #[test]
    fn test_parse_term_nested_function() {
        assert_eq!(
            parse_term("f(g(x))").unwrap(),
            Term::Fn(
                "f".into(),
                vec![Term::Fn("g".into(), vec![Term::Var("x".into())])]
            )
        );
    }

    #[test]
    fn test_parse_term_parenthesized() {
        assert_eq!(parse_term("(x)").unwrap(), Term::Var("x".into()));
    }

    // --- parse_formula ---

    #[test]
    fn test_parse_formula_equality() {
        assert_eq!(
            parse_formula("x = y").unwrap(),
            Formula::Eq(Term::Var("x".into()), Term::Var("y".into()))
        );
    }

    #[test]
    fn test_parse_formula_predicate_no_arg() {
        assert_eq!(
            parse_formula("P").unwrap(),
            Formula::Atom("P".into(), vec![])
        );
    }

    #[test]
    fn test_parse_formula_predicate_one_arg() {
        assert_eq!(
            parse_formula("P x").unwrap(),
            Formula::Atom("P".into(), vec![Term::Var("x".into())])
        );
    }

    #[test]
    fn test_parse_formula_negation() {
        assert_eq!(
            parse_formula("¬P").unwrap(),
            Formula::Not(Box::new(Formula::Atom("P".into(), vec![])))
        );
    }

    #[test]
    fn test_parse_formula_implication() {
        assert_eq!(
            parse_formula("P → Q").unwrap(),
            Formula::To(
                Box::new(Formula::Atom("P".into(), vec![])),
                Box::new(Formula::Atom("Q".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_disjunction() {
        assert_eq!(
            parse_formula("P ∨ Q").unwrap(),
            Formula::Or(
                Box::new(Formula::Atom("P".into(), vec![])),
                Box::new(Formula::Atom("Q".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_conjunction() {
        assert_eq!(
            parse_formula("P ∧ Q").unwrap(),
            Formula::And(
                Box::new(Formula::Atom("P".into(), vec![])),
                Box::new(Formula::Atom("Q".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_iff() {
        assert_eq!(
            parse_formula("P ↔ Q").unwrap(),
            Formula::Iff(
                Box::new(Formula::Atom("P".into(), vec![])),
                Box::new(Formula::Atom("Q".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_forall_single() {
        assert_eq!(
            parse_formula("∀ x, P x").unwrap(),
            Formula::All {
                v: "x".into(),
                sort: Sort::Obj,
                body: Box::new(Formula::Atom("P".into(), vec![Term::Bound(0)]))
            }
        );
    }

    #[test]
    fn test_parse_formula_forall_multi_vars() {
        let result = parse_formula("∀ x y z, P x y z").unwrap();
        let expected = Formula::All {
            v: "x".into(),
            sort: Sort::Obj,
            body: Box::new(Formula::All {
                v: "y".into(),
                sort: Sort::Obj,
                body: Box::new(Formula::All {
                    v: "z".into(),
                    sort: Sort::Obj,
                    body: Box::new(Formula::Atom(
                        "P".into(),
                        vec![Term::Bound(2), Term::Bound(1), Term::Bound(0)],
                    )),
                }),
            }),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_formula_forall_typed_single() {
        assert_eq!(
            parse_formula("∀ (x : N), P x").unwrap(),
            Formula::All {
                v: "x".into(),
                sort: Sort::Nat,
                body: Box::new(Formula::Atom("P".into(), vec![Term::Bound(0)]))
            }
        );
    }

    #[test]
    fn test_parse_formula_forall_typed_group() {
        let result = parse_formula("∀ (x y z : N), P x y z").unwrap();
        let expected = Formula::All {
            v: "x".into(),
            sort: Sort::Nat,
            body: Box::new(Formula::All {
                v: "y".into(),
                sort: Sort::Nat,
                body: Box::new(Formula::All {
                    v: "z".into(),
                    sort: Sort::Nat,
                    body: Box::new(Formula::Atom(
                        "P".into(),
                        vec![Term::Bound(2), Term::Bound(1), Term::Bound(0)],
                    )),
                }),
            }),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_formula_forall_multi_typed_groups() {
        let result = parse_formula("∀ (x : N) (y : Nat), P x y").unwrap();
        let expected = Formula::All {
            v: "x".into(),
            sort: Sort::Nat,
            body: Box::new(Formula::All {
                v: "y".into(),
                sort: Sort::Nat,
                body: Box::new(Formula::Atom(
                    "P".into(),
                    vec![Term::Bound(1), Term::Bound(0)],
                )),
            }),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_formula_exists_single() {
        assert_eq!(
            parse_formula("∃ x, P x").unwrap(),
            Formula::Ex {
                v: "x".into(),
                sort: Sort::Obj,
                body: Box::new(Formula::Atom("P".into(), vec![Term::Bound(0)]))
            }
        );
    }

    #[test]
    fn test_parse_formula_exists_multi_vars() {
        let result = parse_formula("∃ x y z, P x y z").unwrap();
        let expected = Formula::Ex {
            v: "x".into(),
            sort: Sort::Obj,
            body: Box::new(Formula::Ex {
                v: "y".into(),
                sort: Sort::Obj,
                body: Box::new(Formula::Ex {
                    v: "z".into(),
                    sort: Sort::Obj,
                    body: Box::new(Formula::Atom(
                        "P".into(),
                        vec![Term::Bound(2), Term::Bound(1), Term::Bound(0)],
                    )),
                }),
            }),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_formula_nested_forall() {
        let result = parse_formula("∀ x, ∀ y, P x y").unwrap();
        let expected = Formula::All {
            v: "x".into(),
            sort: Sort::Obj,
            body: Box::new(Formula::All {
                v: "y".into(),
                sort: Sort::Obj,
                body: Box::new(Formula::Atom(
                    "P".into(),
                    vec![Term::Bound(1), Term::Bound(0)],
                )),
            }),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_formula_forall_multi_typed_groups_mixed() {
        let result = parse_formula("∀ (x : N) (y : Obj), P x y").unwrap();
        let expected = Formula::All {
            v: "x".into(),
            sort: Sort::Nat,
            body: Box::new(Formula::All {
                v: "y".into(),
                sort: Sort::Obj,
                body: Box::new(Formula::Atom(
                    "P".into(),
                    vec![Term::Bound(1), Term::Bound(0)],
                )),
            }),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_formula_precedence_and_vs_or() {
        let result = parse_formula("P ∧ Q ∨ R").unwrap();
        // ∧ binds tighter than ∨, so (P ∧ Q) ∨ R
        assert_eq!(
            result,
            Formula::Or(
                Box::new(Formula::And(
                    Box::new(Formula::Atom("P".into(), vec![])),
                    Box::new(Formula::Atom("Q".into(), vec![]))
                )),
                Box::new(Formula::Atom("R".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_precedence_to_vs_or() {
        let result = parse_formula("P ∨ Q → R").unwrap();
        // ∨ binds tighter than →, so (P ∨ Q) → R
        assert_eq!(
            result,
            Formula::To(
                Box::new(Formula::Or(
                    Box::new(Formula::Atom("P".into(), vec![])),
                    Box::new(Formula::Atom("Q".into(), vec![]))
                )),
                Box::new(Formula::Atom("R".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_parentheses_override_precedence() {
        let result = parse_formula("P → (Q ∧ R)").unwrap();
        assert_eq!(
            result,
            Formula::To(
                Box::new(Formula::Atom("P".into(), vec![])),
                Box::new(Formula::And(
                    Box::new(Formula::Atom("Q".into(), vec![])),
                    Box::new(Formula::Atom("R".into(), vec![]))
                ))
            )
        );
    }

    /// ∀ は最も優先順位が低い（スコープが右に伸びる）: ∀ x, (P x → Q)
    #[test]
    fn test_parse_formula_forall_scopes_over_to() {
        let result = parse_formula("∀ x, P x → Q").unwrap();
        assert_eq!(
            result,
            Formula::All {
                v: "x".into(),
                sort: Sort::Obj,
                body: Box::new(Formula::To(
                    Box::new(Formula::Atom("P".into(), vec![Term::Bound(0)])),
                    Box::new(Formula::Atom("Q".into(), vec![]))
                ))
            }
        );
    }

    /// ∃ は最も優先順位が低い: ∃ x, (P x ∧ Q)
    #[test]
    fn test_parse_formula_exists_scopes_over_and() {
        let result = parse_formula("∃ x, P x ∧ Q").unwrap();
        assert_eq!(
            result,
            Formula::Ex {
                v: "x".into(),
                sort: Sort::Obj,
                body: Box::new(Formula::And(
                    Box::new(Formula::Atom("P".into(), vec![Term::Bound(0)])),
                    Box::new(Formula::Atom("Q".into(), vec![]))
                ))
            }
        );
    }

    /// → は右結合: P → Q → R = P → (Q → R)
    #[test]
    fn test_parse_formula_to_right_assoc() {
        let result = parse_formula("P → Q → R").unwrap();
        assert_eq!(
            result,
            Formula::To(
                Box::new(Formula::Atom("P".into(), vec![])),
                Box::new(Formula::To(
                    Box::new(Formula::Atom("Q".into(), vec![])),
                    Box::new(Formula::Atom("R".into(), vec![]))
                ))
            )
        );
    }

    /// ∧ は右結合: P ∧ Q ∧ R = P ∧ (Q ∧ R)
    #[test]
    fn test_parse_formula_and_right_assoc() {
        let result = parse_formula("P ∧ Q ∧ R").unwrap();
        assert_eq!(
            result,
            Formula::And(
                Box::new(Formula::Atom("P".into(), vec![])),
                Box::new(Formula::And(
                    Box::new(Formula::Atom("Q".into(), vec![])),
                    Box::new(Formula::Atom("R".into(), vec![]))
                ))
            )
        );
    }

    /// ∨ は右結合: P ∨ Q ∨ R = P ∨ (Q ∨ R)
    #[test]
    fn test_parse_formula_or_right_assoc() {
        let result = parse_formula("P ∨ Q ∨ R").unwrap();
        assert_eq!(
            result,
            Formula::Or(
                Box::new(Formula::Atom("P".into(), vec![])),
                Box::new(Formula::Or(
                    Box::new(Formula::Atom("Q".into(), vec![])),
                    Box::new(Formula::Atom("R".into(), vec![]))
                ))
            )
        );
    }

    /// ⊤ は False → False としてパースされる。
    #[test]
    fn test_parse_formula_true() {
        assert_eq!(
            parse_formula("true").unwrap(),
            Formula::To(Box::new(Formula::False), Box::new(Formula::False))
        );
    }

    #[test]
    fn test_parse_formula_false() {
        assert_eq!(parse_formula("false").unwrap(), Formula::False);
    }

    // --- parse_goal ---

    #[test]
    fn test_parse_goal_empty_hypotheses() {
        let (hyps, target) = parse_goal("⊢ x = y").unwrap();
        assert!(hyps.is_empty());
        assert_eq!(
            target,
            Formula::Eq(Term::Var("x".into()), Term::Var("y".into()))
        );
    }

    #[test]
    fn test_parse_goal_single_hypothesis() {
        let (hyps, target) = parse_goal("P ⊢ Q").unwrap();
        assert_eq!(hyps, vec![Formula::Atom("P".into(), vec![])]);
        assert_eq!(target, Formula::Atom("Q".into(), vec![]));
    }

    #[test]
    fn test_parse_goal_two_hypotheses() {
        let (hyps, target) = parse_goal("P x, Q y ⊢ x = y").unwrap();
        assert_eq!(
            hyps,
            vec![
                Formula::Atom("P".into(), vec![Term::Var("x".into())]),
                Formula::Atom("Q".into(), vec![Term::Var("y".into())]),
            ]
        );
        assert_eq!(
            target,
            Formula::Eq(Term::Var("x".into()), Term::Var("y".into()))
        );
    }

    // --- error cases ---

    #[test]
    fn test_parse_formula_mismatched_parentheses() {
        let err = parse_formula("(P").unwrap_err();
        assert!(matches!(err, Error::Parentheses { lp: 1, rp: 0 }));
    }

    #[test]
    fn test_parse_goal_too_many_targets() {
        let err = parse_goal("P ⊢ Q, R").unwrap_err();
        assert!(matches!(err, Error::TargetCount));
    }

    #[test]
    fn test_parse_term_garbage() {
        let err = parse_term("@#$").unwrap_err();
        assert!(matches!(err, Error::Peg { .. }));
    }

    #[test]
    fn test_parse_formula_garbage() {
        let err = parse_formula("???").unwrap_err();
        assert!(matches!(err, Error::Peg { .. }));
    }
}
