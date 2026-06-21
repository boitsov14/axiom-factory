use crate::syntax::{Formula, Id, Term};
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
    parser::term(&s).map_err(|e| Error::Peg {
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
    let mut formula = parser::formula(&s).map_err(|e| Error::Peg {
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
    let (mut hypotheses, targets) = parser::sequent(&s).map_err(|e| Error::Peg {
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
        p:$pred_id() _ "(" _ ts:(term() ++ (_ "," _)) _ ")" { Formula::Atom(p.into(), ts) } /
        p:$pred_id() { Formula::Atom(p.into(), vec![]) }

    pub rule formula() -> Formula = precedence!{
        p:@ _ iff() _ q:(@) { Formula::Iff(Box::new(p), Box::new(q)) }
        --
        p:@ _ to() _ q:(@) { Formula::To(Box::new(p), Box::new(q)) }
        --
        p:@ _ or() _ q:(@) { Formula::Or(Box::new(p), Box::new(q)) }
        --
        p:@ _ and() _ q:(@) { Formula::And(Box::new(p), Box::new(q)) }
        --
        not() _ p:@ { Formula::Not(Box::new(p)) }
        all() _ vs:(qvar() ++ (_ "," _)) _ comma()? _ p:@ {
            vs.into_iter().rev().fold(p, |p, (v, sort)| Formula::All { v, sort, body: Box::new(p) })
        }
        ex() _ vs:(qvar() ++ (_ "," _)) _ comma()? _ p:@ {
            vs.into_iter().rev().fold(p, |p, (v, sort)| Formula::Ex { v, sort, body: Box::new(p) })
        }
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

    rule qvar() -> (Id, Sort) =
        v:$bdd_var_id() _ ":" _ s:sort() { (v.into(), s) } /
        v:$bdd_var_id() { (v.into(), Sort::Obj) }

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
