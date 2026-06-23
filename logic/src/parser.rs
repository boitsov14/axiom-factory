use crate::syntax::{
    Formula,
    Formula::*,
    Id,
    Sort,
    Sort::*,
    Term,
    Term::{Bound, Var},
};
use thiserror::Error;

/// Parser Error
#[derive(Error, Debug)]
pub enum Error {
    /// 括弧の数が一致しない
    #[error("Found {lp} left parentheses and {rp} right parentheses.")]
    Parentheses { lp: usize, rp: usize },
    /// Peg Error
    #[error("
 |
 | {s}
 | {}^___
 |
 = expected {}", " ".repeat(e.location.column - 1), e.expected)]
    Peg {
        s: String,
        e: peg::error::ParseError<peg::str::LineCol>,
    },
}

/// `Term` を Parse
pub fn parse_term(s: &str) -> Result<Term, Error> {
    check_parentheses(s)?;
    parser::term(s).map_err(|e| Error::Peg { s: s.to_owned(), e })
}

/// `Formula` を Parse
pub fn parse_formula(s: &str) -> Result<Formula, Error> {
    check_parentheses(s)?;
    let mut fml = parser::formula(s).map_err(|e| Error::Peg { s: s.to_owned(), e })?;
    close_formula(&mut fml);
    Ok(fml)
}

/// 括弧の数が一致するか確認
fn check_parentheses(s: &str) -> Result<(), Error> {
    let lp = s.chars().filter(|&c| c == '(').count();
    let rp = s.chars().filter(|&c| c == ')').count();
    if lp == rp {
        Ok(())
    } else {
        Err(Error::Parentheses { lp, rp })
    }
}

/// 現在の束縛変数スタックに従って `Term` を閉じる。
fn close_term_at(t: &mut Term, stack: &[Id]) {
    match t {
        Var(x) => {
            if let Some(i) = stack.iter().rev().position(|v| v == x) {
                *t = Bound(i);
            }
        }
        Bound(_) => {}
        Term::Fn(_, args) => {
            for u in args {
                close_term_at(u, stack);
            }
        }
    }
}

/// 束縛変数名を de Bruijn index に変換する。
fn close_formula(fml: &mut Formula) {
    close_formula_at(fml, &mut vec![]);
}

/// 現在の束縛変数スタックに従って `Formula` を閉じる。
fn close_formula_at(fml: &mut Formula, stack: &mut Vec<Id>) {
    match fml {
        False => {}
        Atom(_, args) => {
            for t in args {
                close_term_at(t, stack);
            }
        }
        Eq(t, u) => {
            close_term_at(t, stack);
            close_term_at(u, stack);
        }
        Not(p) => close_formula_at(p, stack),
        And(p, q) | Or(p, q) | To(p, q) | Iff(p, q) => {
            close_formula_at(p, stack);
            close_formula_at(q, stack);
        }
        All { v, body, .. } | Ex { v, body, .. } => {
            stack.push(v.clone());
            close_formula_at(body, stack);
            stack.pop();
        }
    }
}

peg::parser!(grammar parser() for str {
    /// `Term` を Parse
    pub rule term() -> Term = quiet!{
        f:$func_id() _ "(" _ ts:(term() ++ (_ "," _)) _ ")" { Term::Fn(f.into(), ts) } /
        v:$var_id() { Var(v.into()) } /
        "(" _ t:term() _ ")" { t }
    } / expected!("term")

    rule atom() -> Formula =
        p_false() { False } /
        s:term() _ eq() _ t:term() { Eq(s, t) } /
        p:$pred_id() _ "(" _ ts:(term() ++ (_ "," _)) _ ")" { Atom(p.into(), ts) } /
        p:$pred_id() { Atom(p.into(), vec![]) }

    /// 論理式を構文解析する。
    ///
    /// すべての演算子は右結合である。
    ///
    /// 優先順位: ¬, ∀, ∃ > ∧ > ∨ > → > ↔
    pub rule formula() -> Formula = precedence!{
        p:@ _ iff() _ q:(@) { Iff(Box::new(p), Box::new(q)) }
        --
        p:@ _ to() _ q:(@) { To(Box::new(p), Box::new(q)) }
        --
        p:@ _ or() _ q:(@) { Or(Box::new(p), Box::new(q)) }
        --
        p:@ _ and() _ q:(@) { And(Box::new(p), Box::new(q)) }
        --
        not() _ p:@ { Not(Box::new(p)) }
        all() _ v:$var_id() _ ":" _ s:sort() _ p:@ { All { v: v.into(), sort: s, body: Box::new(p) } }
        all() _ v:$var_id() _ p:@ { All { v: v.into(), sort: Obj, body: Box::new(p) } }
        ex() _ v:$var_id() _ ":" _ s:sort() _ p:@ { Ex { v: v.into(), sort: s, body: Box::new(p) } }
        ex() _ v:$var_id() _ p:@ { Ex { v: v.into(), sort: Obj, body: Box::new(p) } }
        --
        p:atom() { p }
        "(" _ p:formula() _ ")" { p }
    } / expected!("formula")

    rule sort() -> Sort = quiet!{
        "Obj" { Obj } /
        "V" { Obj } /
        "Nat" { Nat } /
        "N" { Nat } /
        r"\mathbb{N}" { Nat } /
        "Int" { Int } /
        "Z" { Int } /
        r"\mathbb{Z}" { Int } /
        "Rat" { Rat } /
        "Q" { Rat } /
        r"\mathbb{Q}" { Rat }
    } / expected!("sort")

    rule alpha() = ['a'..='z' | 'A'..='Z']
    rule digit() = ['0'..='9' | '_' | '\'']
    rule id() = alpha() (alpha() / digit())*
    rule var_id() = id()
    rule func_id() = id()
    rule pred_id() = quiet!{ !"true" id() } / expected!("predicate")
    rule p_false() = quiet!{ "⊥" / "⟂" / "false" / r"\bot" }
    rule not() = quiet!{ "¬" / "~" / "not" / r"\lnot" / r"\neg" } / expected!(r#""¬""#)
    rule and() = quiet!{ "∧" / r"/\" / "&" / "and" / r"\land" / r"\wedge" } / expected!(r#""∧""#)
    rule or() = quiet!{ "∨" / r"\/" / "|" / "or" / r"\lor" / r"\vee" } / expected!(r#""∨""#)
    rule to() = quiet!{ "→" / "->" / "=>" / "to" / r"\rightarrow" / r"\to" } / expected!(r#""→""#)
    rule iff() = quiet!{ "↔" / "<->" / "<=>" / "iff" / r"\leftrightarrow" } / expected!(r#""↔""#)
    rule all() = quiet!{ "∀" / "!" / "all" / r"\forall" } / expected!(r#""∀""#)
    rule ex() = quiet!{ "∃" / "?" / "ex" / r"\exists" } / expected!(r#""∃""#)
    rule eq() = quiet!{ "=" } / expected!(r#""=""#)
    rule _ = quiet!{ [' ']* }
});

#[cfg(test)]
mod tests {
    use super::*;
    use Term::Fn;

    // --- parse_term ---

    #[test]
    fn test_parse_term_variable() {
        assert_eq!(parse_term("x").unwrap(), Var("x".into()));
    }

    #[test]
    fn test_parse_term_unary_function() {
        assert_eq!(
            parse_term("f(x)").unwrap(),
            Fn("f".into(), vec![Var("x".into())])
        );
    }

    #[test]
    fn test_parse_term_binary_function() {
        assert_eq!(
            parse_term("f(x,y)").unwrap(),
            Fn("f".into(), vec![Var("x".into()), Var("y".into())])
        );
    }

    #[test]
    fn test_parse_term_nested_function() {
        assert_eq!(
            parse_term("f(g(x))").unwrap(),
            Fn("f".into(), vec![Fn("g".into(), vec![Var("x".into())])])
        );
    }

    #[test]
    fn test_parse_term_parenthesized() {
        assert_eq!(parse_term("(x)").unwrap(), Var("x".into()));
    }

    #[test]
    fn test_parse_term_with_apostrophe() {
        assert_eq!(parse_term("x'").unwrap(), Var("x'".into()));
        assert_eq!(
            parse_term("f(x')").unwrap(),
            Fn("f".into(), vec![Var("x'".into())])
        );
    }

    // --- parse_formula ---

    #[test]
    fn test_parse_formula_equality() {
        assert_eq!(
            parse_formula("x = y").unwrap(),
            Eq(Var("x".into()), Var("y".into()))
        );
    }

    #[test]
    fn test_parse_formula_predicate_no_arg() {
        assert_eq!(parse_formula("P").unwrap(), Atom("P".into(), vec![]));
    }

    #[test]
    fn test_parse_formula_predicate_one_arg() {
        assert_eq!(
            parse_formula("P(x)").unwrap(),
            Atom("P".into(), vec![Var("x".into())])
        );
    }

    #[test]
    fn test_parse_formula_predicate_two_args() {
        assert_eq!(
            parse_formula("P(x,y)").unwrap(),
            Atom("P".into(), vec![Var("x".into()), Var("y".into())])
        );
    }

    #[test]
    fn test_parse_formula_equality_with_function() {
        assert_eq!(
            parse_formula("f(x) = g(y)").unwrap(),
            Eq(
                Fn("f".into(), vec![Var("x".into())]),
                Fn("g".into(), vec![Var("y".into())])
            )
        );
    }

    #[test]
    fn test_parse_formula_negation() {
        assert_eq!(
            parse_formula("¬P").unwrap(),
            Not(Box::new(Atom("P".into(), vec![])))
        );
    }

    #[test]
    fn test_parse_formula_double_negation() {
        let result = parse_formula("¬¬P").unwrap();
        assert_eq!(
            result,
            Not(Box::new(Not(Box::new(Atom("P".into(), vec![])))))
        );
    }

    #[test]
    fn test_parse_formula_implication() {
        assert_eq!(
            parse_formula("P → Q").unwrap(),
            To(
                Box::new(Atom("P".into(), vec![])),
                Box::new(Atom("Q".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_disjunction() {
        assert_eq!(
            parse_formula("P ∨ Q").unwrap(),
            Or(
                Box::new(Atom("P".into(), vec![])),
                Box::new(Atom("Q".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_conjunction() {
        assert_eq!(
            parse_formula("P ∧ Q").unwrap(),
            And(
                Box::new(Atom("P".into(), vec![])),
                Box::new(Atom("Q".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_iff() {
        assert_eq!(
            parse_formula("P ↔ Q").unwrap(),
            Iff(
                Box::new(Atom("P".into(), vec![])),
                Box::new(Atom("Q".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_implication_chain_right_assoc() {
        let result = parse_formula("P → Q → R").unwrap();
        assert_eq!(
            result,
            To(
                Box::new(Atom("P".into(), vec![])),
                Box::new(To(
                    Box::new(Atom("Q".into(), vec![])),
                    Box::new(Atom("R".into(), vec![]))
                ))
            )
        );
    }

    #[test]
    fn test_parse_formula_iff_chain_right_assoc() {
        let result = parse_formula("P ↔ Q ↔ R").unwrap();
        assert_eq!(
            result,
            Iff(
                Box::new(Atom("P".into(), vec![])),
                Box::new(Iff(
                    Box::new(Atom("Q".into(), vec![])),
                    Box::new(Atom("R".into(), vec![]))
                ))
            )
        );
    }

    #[test]
    fn test_parse_formula_and_chain_right_assoc() {
        let result = parse_formula("P ∧ Q ∧ R").unwrap();
        assert_eq!(
            result,
            And(
                Box::new(Atom("P".into(), vec![])),
                Box::new(And(
                    Box::new(Atom("Q".into(), vec![])),
                    Box::new(Atom("R".into(), vec![]))
                ))
            )
        );
    }

    #[test]
    fn test_parse_formula_or_chain_right_assoc() {
        let result = parse_formula("P ∨ Q ∨ R").unwrap();
        assert_eq!(
            result,
            Or(
                Box::new(Atom("P".into(), vec![])),
                Box::new(Or(
                    Box::new(Atom("Q".into(), vec![])),
                    Box::new(Atom("R".into(), vec![]))
                ))
            )
        );
    }

    // precedence: ¬, ∀, ∃ > ∧ > ∨ > → > ↔

    #[test]
    fn test_parse_formula_negation_binds_tighter_than_and() {
        // ¬P ∧ Q  = (¬P) ∧ Q
        let result = parse_formula("¬P ∧ Q").unwrap();
        assert_eq!(
            result,
            And(
                Box::new(Not(Box::new(Atom("P".into(), vec![])))),
                Box::new(Atom("Q".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_and_binds_tighter_than_or() {
        // P ∧ Q ∨ R = (P ∧ Q) ∨ R
        let result = parse_formula("P ∧ Q ∨ R").unwrap();
        assert_eq!(
            result,
            Or(
                Box::new(And(
                    Box::new(Atom("P".into(), vec![])),
                    Box::new(Atom("Q".into(), vec![]))
                )),
                Box::new(Atom("R".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_or_binds_tighter_than_to() {
        // P ∨ Q → R = (P ∨ Q) → R
        let result = parse_formula("P ∨ Q → R").unwrap();
        assert_eq!(
            result,
            To(
                Box::new(Or(
                    Box::new(Atom("P".into(), vec![])),
                    Box::new(Atom("Q".into(), vec![]))
                )),
                Box::new(Atom("R".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_to_binds_tighter_than_iff() {
        // P → Q ↔ R = (P → Q) ↔ R
        let result = parse_formula("P → Q ↔ R").unwrap();
        assert_eq!(
            result,
            Iff(
                Box::new(To(
                    Box::new(Atom("P".into(), vec![])),
                    Box::new(Atom("Q".into(), vec![]))
                )),
                Box::new(Atom("R".into(), vec![]))
            )
        );
    }

    #[test]
    fn test_parse_formula_parentheses_override_precedence() {
        let result = parse_formula("P → (Q ∧ R)").unwrap();
        assert_eq!(
            result,
            To(
                Box::new(Atom("P".into(), vec![])),
                Box::new(And(
                    Box::new(Atom("Q".into(), vec![])),
                    Box::new(Atom("R".into(), vec![]))
                ))
            )
        );
    }

    #[test]
    fn test_parse_formula_negation_over_forall() {
        // ¬∀x P(x) = ¬(∀x P(x))
        let result = parse_formula("¬∀x P(x)").unwrap();
        assert_eq!(
            result,
            Not(Box::new(All {
                v: "x".into(),
                sort: Obj,
                body: Box::new(Atom("P".into(), vec![Bound(0)]))
            }))
        );
    }

    #[test]
    fn test_parse_formula_forall_scopes_over_to() {
        // ∀x P(x) → Q = (∀x P(x)) → Q
        let result = parse_formula("∀x P(x) → Q").unwrap();
        assert_eq!(
            result,
            To(
                Box::new(All {
                    v: "x".into(),
                    sort: Obj,
                    body: Box::new(Atom("P".into(), vec![Bound(0)]))
                }),
                Box::new(Atom("Q".into(), vec![]))
            )
        );
    }

    // Quantifiers

    #[test]
    fn test_parse_formula_forall_single() {
        assert_eq!(
            parse_formula("∀x P(x)").unwrap(),
            All {
                v: "x".into(),
                sort: Obj,
                body: Box::new(Atom("P".into(), vec![Bound(0)]))
            }
        );
    }

    #[test]
    fn test_parse_formula_forall_typed_nat() {
        assert_eq!(
            parse_formula("∀x:Nat P(x)").unwrap(),
            All {
                v: "x".into(),
                sort: Nat,
                body: Box::new(Atom("P".into(), vec![Bound(0)]))
            }
        );
    }

    #[test]
    fn test_parse_formula_forall_double() {
        let result = parse_formula("∀x∀y P(x,y)").unwrap();
        let expected = All {
            v: "x".into(),
            sort: Obj,
            body: Box::new(All {
                v: "y".into(),
                sort: Obj,
                body: Box::new(Atom("P".into(), vec![Bound(1), Bound(0)])),
            }),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_formula_exists_single() {
        assert_eq!(
            parse_formula("∃x P(x)").unwrap(),
            Ex {
                v: "x".into(),
                sort: Obj,
                body: Box::new(Atom("P".into(), vec![Bound(0)]))
            }
        );
    }

    #[test]
    fn test_parse_formula_exists_typed() {
        assert_eq!(
            parse_formula("∃x:Int P(x)").unwrap(),
            Ex {
                v: "x".into(),
                sort: Int,
                body: Box::new(Atom("P".into(), vec![Bound(0)]))
            }
        );
    }

    #[test]
    fn test_parse_formula_exists_double() {
        let result = parse_formula("∃x∃y P(x,y)").unwrap();
        assert_eq!(
            result,
            Ex {
                v: "x".into(),
                sort: Obj,
                body: Box::new(Ex {
                    v: "y".into(),
                    sort: Obj,
                    body: Box::new(Atom("P".into(), vec![Bound(1), Bound(0)]))
                })
            }
        );
    }

    #[test]
    fn test_parse_formula_mixed_quantifiers() {
        let result = parse_formula("∀x∃y P(x,y)").unwrap();
        assert_eq!(
            result,
            All {
                v: "x".into(),
                sort: Obj,
                body: Box::new(Ex {
                    v: "y".into(),
                    sort: Obj,
                    body: Box::new(Atom("P".into(), vec![Bound(1), Bound(0)]))
                })
            }
        );
    }

    #[test]
    fn test_parse_formula_forall_nat_exists_int() {
        let result = parse_formula("∀x:N ∃y:Int P(x,y)").unwrap();
        assert_eq!(
            result,
            All {
                v: "x".into(),
                sort: Nat,
                body: Box::new(Ex {
                    v: "y".into(),
                    sort: Int,
                    body: Box::new(Atom("P".into(), vec![Bound(1), Bound(0)]))
                })
            }
        );
    }

    #[test]
    fn test_parse_formula_false() {
        assert_eq!(parse_formula("false").unwrap(), False);
    }

    #[test]
    fn test_parse_formula_bot_as_false() {
        assert_eq!(parse_formula("⊥").unwrap(), False);
    }

    // Error cases

    #[test]
    fn test_parse_formula_mismatched_parentheses() {
        let err = parse_formula("(P").unwrap_err();
        assert!(matches!(err, Error::Parentheses { lp: 1, rp: 0 }));
    }

    #[test]
    fn test_parse_formula_mismatched_parentheses_right() {
        let err = parse_formula("P)").unwrap_err();
        assert!(matches!(err, Error::Parentheses { lp: 0, rp: 1 }));
    }

    #[test]
    fn test_parse_formula_invalid_formula() {
        let err = parse_formula("P ∧").unwrap_err();
        assert!(matches!(err, Error::Peg { .. }));
    }

    // de Bruijn index verification

    #[test]
    fn test_parse_formula_nested_quantifier_bruijn() {
        // ∀x∀y P(x,y) → Bound(1)=x, Bound(0)=y
        let result = parse_formula("∀x∀y P(x,y)").unwrap();
        let All { body, .. } = result else {
            unreachable!();
        };
        let All { body: inner, .. } = *body else {
            unreachable!();
        };
        let Atom(_, args) = *inner else {
            unreachable!();
        };
        assert_eq!(args[0], Bound(1)); // x is outer
        assert_eq!(args[1], Bound(0)); // y is inner
    }
}
