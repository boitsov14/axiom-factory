use crate::syntax::{Formula, Formula::*, Goal, Term};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tactic {
    Intro,
    Assumption { hyp: usize },
    Apply { hyp: usize },
    Constructor,
    Left,
    Right,
    Cases { hyp: usize },
    Exists { term: Term },
    Specialize { hyp: usize, arg: Arg },
    Have { formula: Formula },
    Exfalso,
    ByContra,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arg {
    Term(Term),
    Hyp(usize),
}

impl Tactic {
    /// `Goal` にタクティクを適用する。
    ///
    /// # Errors
    ///
    /// 対象ゴールに対してタクティクが適用できない場合にエラーを返す。
    pub fn apply(&self, goal: Goal) -> Result<Vec<Goal>, String> {
        use Tactic::*;
        match self {
            Intro => intro(goal),
            Assumption { hyp } => assumption(&goal, *hyp),
            Apply { hyp } => apply_hyp(goal, *hyp),
            Constructor => constructor(goal),
            Left => left(goal),
            Right => right(goal),
            Cases { hyp } => cases(goal, *hyp),
            Exists { term } => exists(goal, term),
            Specialize { hyp, arg } => specialize(goal, *hyp, arg),
            Have { formula } => Ok(have(goal, formula.clone())),
            Exfalso => exfalso(goal),
            ByContra => by_contra(goal),
        }
    }
}

/// 導入タクティクを実行する。
fn intro(mut goal: Goal) -> Result<Vec<Goal>, String> {
    match goal.target {
        To(p, q) => {
            goal.hypotheses.push(*p);
            goal.target = *q;
            Ok(vec![goal])
        }
        All { v, body, .. } => {
            let mut p = *body;
            p.open(&Term::Var(v));
            goal.target = p;
            Ok(vec![goal])
        }
        Not(p) => {
            goal.hypotheses.push(*p);
            goal.target = False;
            Ok(vec![goal])
        }
        _ => Err("intro cannot be applied".into()),
    }
}

/// 指定された仮定でゴールを閉じる。
fn assumption(goal: &Goal, hyp: usize) -> Result<Vec<Goal>, String> {
    if goal.hypotheses.get(hyp).is_some_and(|p| p == &goal.target) {
        Ok(vec![])
    } else {
        Err("the selected hypothesis does not match the target".into())
    }
}

/// 含意・否定・同値の仮定を適用する。
fn apply_hyp(mut goal: Goal, hyp: usize) -> Result<Vec<Goal>, String> {
    let Some(formula) = goal.hypotheses.get(hyp).cloned() else {
        return Err("selected hypothesis does not exist".into());
    };

    match formula {
        To(p, q) if *q == goal.target => {
            goal.target = *p;
            Ok(vec![goal])
        }
        Not(p) if goal.target == False => {
            goal.target = *p;
            Ok(vec![goal])
        }
        Iff(p, q) if *q == goal.target => {
            goal.target = *p;
            Ok(vec![goal])
        }
        Iff(p, q) if *p == goal.target => {
            goal.target = *q;
            Ok(vec![goal])
        }
        _ => Err("apply cannot be applied".into()),
    }
}

/// 連言または同値のゴールを分割する。
fn constructor(goal: Goal) -> Result<Vec<Goal>, String> {
    match goal.target {
        And(p, q) | Iff(p, q) => Ok(vec![
            Goal {
                hypotheses: goal.hypotheses.clone(),
                target: *p,
            },
            Goal {
                hypotheses: goal.hypotheses,
                target: *q,
            },
        ]),
        _ => Err("constructor cannot be applied".into()),
    }
}

/// 選言ゴールの左側を選ぶ。
fn left(mut goal: Goal) -> Result<Vec<Goal>, String> {
    match goal.target {
        Or(p, _) => {
            goal.target = *p;
            Ok(vec![goal])
        }
        _ => Err("left cannot be applied".into()),
    }
}

/// 選言ゴールの右側を選ぶ。
fn right(mut goal: Goal) -> Result<Vec<Goal>, String> {
    match goal.target {
        Or(_, q) => {
            goal.target = *q;
            Ok(vec![goal])
        }
        _ => Err("right cannot be applied".into()),
    }
}

/// 仮定を分解または場合分けする。
fn cases(goal: Goal, hyp: usize) -> Result<Vec<Goal>, String> {
    let Some(formula) = goal.hypotheses.get(hyp).cloned() else {
        return Err("selected hypothesis does not exist".into());
    };

    match formula {
        And(p, q) | Iff(p, q) => {
            let mut next = goal;
            next.hypotheses.remove(hyp);
            next.hypotheses.push(*p);
            next.hypotheses.push(*q);
            Ok(vec![next])
        }
        Or(p, q) => {
            let mut l = goal.clone();
            l.hypotheses.remove(hyp);
            l.hypotheses.push(*p);

            let mut r = goal;
            r.hypotheses.remove(hyp);
            r.hypotheses.push(*q);
            Ok(vec![l, r])
        }
        Ex { v, body, .. } => {
            let mut p = *body;
            p.open(&Term::Var(v));
            let mut next = goal;
            next.hypotheses.remove(hyp);
            next.hypotheses.push(p);
            Ok(vec![next])
        }
        False => Ok(vec![]),
        _ => Err("cases cannot be applied".into()),
    }
}

/// 存在ゴールに witness を与える。
fn exists(mut goal: Goal, term: &Term) -> Result<Vec<Goal>, String> {
    match goal.target {
        Ex { body, .. } => {
            let mut p = *body;
            p.open(term);
            goal.target = p;
            Ok(vec![goal])
        }
        _ => Err("exists cannot be applied".into()),
    }
}

/// 全称または含意の仮定を具体化する。
fn specialize(mut goal: Goal, hyp: usize, arg: &Arg) -> Result<Vec<Goal>, String> {
    let Some(formula) = goal.hypotheses.get(hyp).cloned() else {
        return Err("selected hypothesis does not exist".into());
    };

    match (formula, arg) {
        (All { body, .. }, Arg::Term(t)) => {
            let mut p = *body;
            p.open(t);
            goal.hypotheses.push(p);
            Ok(vec![goal])
        }
        (To(p, q), Arg::Hyp(arg_hyp))
            if goal.hypotheses.get(*arg_hyp).is_some_and(|h| h == &*p) =>
        {
            goal.hypotheses.push(*q);
            Ok(vec![goal])
        }
        _ => Err("specialize cannot be applied".into()),
    }
}

/// 中間命題を追加する。
fn have(goal: Goal, formula: Formula) -> Vec<Goal> {
    let mut after = goal.clone();
    after.hypotheses.push(formula.clone());
    vec![
        Goal {
            hypotheses: goal.hypotheses,
            target: formula,
        },
        after,
    ]
}

/// ゴールを `False` に変える。
fn exfalso(mut goal: Goal) -> Result<Vec<Goal>, String> {
    if goal.target == False {
        Err("exfalso is already the target".into())
    } else {
        goal.target = False;
        Ok(vec![goal])
    }
}

/// 背理法でゴールを `False` に変える。
fn by_contra(mut goal: Goal) -> Result<Vec<Goal>, String> {
    if goal.target == False {
        Err("by_contra cannot be applied to False".into())
    } else {
        let p = goal.target;
        goal.hypotheses.push(Not(Box::new(p)));
        goal.target = False;
        Ok(vec![goal])
    }
}
