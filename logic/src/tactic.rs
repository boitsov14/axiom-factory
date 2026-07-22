use crate::{
    ids::fresh,
    syntax::{Formula, Formula::*, Goal, Term, Term::*},
};
use Tactic::*;
use maplit::hashset;

pub enum RewriteDirection {
    Fwd,
    Rev,
}

pub enum RewriteLocation {
    Target,
    Hypothesis { i: usize },
}

pub enum Tactic {
    /// `⊢ ¬P` を `P ⊢ ⊥` に変換
    IntroNot,
    /// `⊢ P → Q` を `P ⊢ Q` に変換
    IntroTo,
    /// `⊢ ∀x P(x)` を `⊢ P(x)` に変換
    IntroAll,
    /// `⊢ P ∧ Q` を `⊢ P` と `⊢ Q` に分割
    ConstructorAnd,
    /// `⊢ P ↔ Q` を `P ⊢ Q` と `Q ⊢ P` に分割
    ConstructorIff,
    /// `⊢ P ∨ Q` を `⊢ P` に変換
    Left,
    /// `⊢ P ∨ Q` を `⊢ Q` に変換
    Right,
    /// `⊢ ∃x P(x)` を `⊢ P(t)` に変換
    Exists { t: Term },
    /// 結論を `⊥` に変更
    Exfalso,
    /// `⊢ P` を `¬P ⊢ ⊥` に変換（背理法）
    ByContra,
    /// `⊢ P` を `Q ⊢ P` と `¬Q ⊢ P` に場合分け
    ByCases { fml: Formula },
    /// 結論または仮説の否定を次の規則で内側へ変形
    /// - `¬¬P` を `P` に変換
    /// - `¬(P ∨ Q)` を `¬P ∧ ¬Q` に変換
    /// - `¬(P ∧ Q)` を `¬P ∨ ¬Q` に変換
    /// - `¬(P → Q)` を `P ∧ ¬Q` に変換
    /// - `¬∃x P(x)` を `∀x ¬P(x)` に変換
    /// - `¬∀x P(x)` を `∃x ¬P(x)` に変換
    PushNot { location: RewriteLocation },
    /// 指定した仮説を削除
    Clear { i: usize },
    /// 仮説との一致、`⊥` 仮説、または矛盾する仮説から証明完了
    Close,
    /// `¬P ⊢ ⊥` を `¬P ⊢ P` に変換
    ApplyNot { i: usize },
    /// `P → Q ⊢ Q` を `P → Q ⊢ P` に変換
    ApplyTo { i: usize },
    /// `P → Q ⊢ R` を `⊢ P` と `Q ⊢ R` に分割
    ForwardTo { i: usize },
    /// 仮説 `P ↔ Q` により、結論または仮説の `P`, `Q` を書き換える
    RewriteIff {
        i: usize,
        direction: RewriteDirection,
        location: RewriteLocation,
    },
    /// `⊢ t = t` を証明完了
    Rfl,
    /// 仮説 `a = b` により、結論または仮説の `a` を `b` に書き換える
    Rw {
        i: usize,
        direction: RewriteDirection,
        location: RewriteLocation,
    },
    /// `⊢ s = u` を `⊢ s = t` と `⊢ t = u` に分割
    Calc { t: Term },
    /// `⊢ f(s, t) = f(u, v)` を `⊢ s = u` と `⊢ t = v` に分割
    Congr,
    /// `P ∧ Q ⊢` を `P, Q ⊢` に分解
    CasesAnd { i: usize },
    /// `P ∨ Q ⊢` を `P ⊢` と `Q ⊢` に場合分け
    CasesOr { i: usize },
    /// `P ↔ Q ⊢` を `P → Q, Q → P ⊢` に分解
    CasesIff { i: usize },
    /// `∃x P(x) ⊢` を `P(x) ⊢` に変換
    CasesEx { i: usize },
    /// `∀x P(x) ⊢` を `∀x P(x), P(t) ⊢` に変換
    SpecializeAll { i: usize, t: Term },
    /// `P → Q, P ⊢` を `P → Q, P, Q ⊢` に変換
    SpecializeTo { i: usize },
    /// 中間命題 `P` を導入し、`⊢ Q` を `⊢ P` と `P ⊢ Q` に分割
    Have { fml: Formula },
    /// 定理を仮説に追加
    UseTheorem { fml: Formula },
}

impl Tactic {
    /// `Goal` にタクティクを適用し、新しいゴールのリストを返す。
    /// `can_apply` で事前に適用可能性を確認すること。
    pub fn apply(&self, goal: &Goal) -> Vec<Goal> {
        assert!(self.can_apply(goal));
        match self {
            // `⊢ ¬P` を `P ⊢ ⊥` に変換
            IntroNot => {
                let Not(p) = &goal.target else { unreachable!() };
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.push(*p.clone());
                        h
                    },
                    target: False,
                }]
            }

            // `⊢ P → Q` を `P ⊢ Q` に変換
            IntroTo => {
                let To(p, q) = &goal.target else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.push(*p.clone());
                        h
                    },
                    target: *q.clone(),
                }]
            }

            // `⊢ ∀x P(x)` を `⊢ P(x)` に変換
            IntroAll => {
                let All { v, body, .. } = &goal.target else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: {
                        let mut used = hashset!();
                        for h in &goal.hypotheses {
                            h.ids(&mut used);
                        }
                        let v = fresh(v, &used);
                        let mut body = *body.clone();
                        body.open(&Var(v));
                        body
                    },
                }]
            }

            // `⊢ P ∧ Q` を `⊢ P` と `⊢ Q` に分割
            ConstructorAnd => {
                let And(p, q) = &goal.target else {
                    unreachable!()
                };
                vec![
                    Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: *p.clone(),
                    },
                    Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: *q.clone(),
                    },
                ]
            }

            // `⊢ P ↔ Q` を `P ⊢ Q` と `Q ⊢ P` に分割
            ConstructorIff => {
                let Iff(p, q) = &goal.target else {
                    unreachable!()
                };
                vec![
                    Goal {
                        hypotheses: {
                            let mut h = goal.hypotheses.clone();
                            h.push(*p.clone());
                            h
                        },
                        target: *q.clone(),
                    },
                    Goal {
                        hypotheses: {
                            let mut h = goal.hypotheses.clone();
                            h.push(*q.clone());
                            h
                        },
                        target: *p.clone(),
                    },
                ]
            }

            // `⊢ P ∨ Q` を `⊢ P` に変換
            Left => {
                let Or(p, _) = &goal.target else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: *p.clone(),
                }]
            }

            // `⊢ P ∨ Q` を `⊢ Q` に変換
            Right => {
                let Or(_, q) = &goal.target else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: *q.clone(),
                }]
            }

            // `⊢ ∃x P(x)` を `⊢ P(t)` に変換
            Exists { t } => {
                let Ex { body, .. } = &goal.target else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: {
                        let mut body = *body.clone();
                        body.open(t);
                        body
                    },
                }]
            }

            // 結論を `⊥` に変更
            Exfalso => {
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: False,
                }]
            }

            // `⊢ P` を `¬P ⊢ ⊥` に変換（背理法）
            ByContra => {
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        let p = goal.target.clone();
                        h.push(Not(Box::new(p)));
                        h
                    },
                    target: False,
                }]
            }

            // `⊢ P` を `Q ⊢ P` と `¬Q ⊢ P` に場合分け
            ByCases { fml } => {
                vec![
                    Goal {
                        hypotheses: {
                            let mut h = goal.hypotheses.clone();
                            h.push(fml.clone());
                            h
                        },
                        target: goal.target.clone(),
                    },
                    Goal {
                        hypotheses: {
                            let mut h = goal.hypotheses.clone();
                            h.push(Not(Box::new(fml.clone())));
                            h
                        },
                        target: goal.target.clone(),
                    },
                ]
            }

            // 結論または仮説の否定を次の規則で内側へ変形
            // - `¬¬P` を `P` に変換
            // - `¬(P ∨ Q)` を `¬P ∧ ¬Q` に変換
            // - `¬(P ∧ Q)` を `¬P ∨ ¬Q` に変換
            // - `¬(P → Q)` を `P ∧ ¬Q` に変換
            // - `¬∃x P(x)` を `∀x ¬P(x)` に変換
            // - `¬∀x P(x)` を `∃x ¬P(x)` に変換
            PushNot { location } => {
                let np = match location {
                    RewriteLocation::Target => &goal.target,
                    RewriteLocation::Hypothesis { i } => &goal.hypotheses[*i],
                };
                let Not(p) = np else { unreachable!() };
                let p = match &**p {
                    Not(p) => *p.clone(),
                    Or(p, q) => And(Box::new(Not(p.clone())), Box::new(Not(q.clone()))),
                    And(p, q) => Or(Box::new(Not(p.clone())), Box::new(Not(q.clone()))),
                    To(p, q) => And(p.clone(), Box::new(Not(q.clone()))),
                    Ex { v, sort, body } => All {
                        v: v.clone(),
                        sort: sort.clone(),
                        body: Box::new(Not(body.clone())),
                    },
                    All { v, sort, body } => Ex {
                        v: v.clone(),
                        sort: sort.clone(),
                        body: Box::new(Not(body.clone())),
                    },
                    _ => unreachable!(),
                };
                let mut goal = goal.clone();
                match location {
                    RewriteLocation::Target => goal.target = p,
                    RewriteLocation::Hypothesis { i } => goal.hypotheses[*i] = p,
                }
                vec![goal]
            }

            // 指定した仮説を削除
            Clear { i } => {
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.remove(*i);
                        h
                    },
                    target: goal.target.clone(),
                }]
            }

            // 仮説と結論の一致、`⊥` 仮説、矛盾する仮説から証明完了
            // `⊢ t = t` を証明完了
            Close | Rfl => {
                vec![]
            }

            // `¬P ⊢ ⊥` を `¬P ⊢ P` に変換
            ApplyNot { i } => {
                let Not(p) = &goal.hypotheses[*i] else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: *p.clone(),
                }]
            }

            // `P → Q ⊢ Q` を `P → Q ⊢ P` に変換
            ApplyTo { i } => {
                let To(p, _) = &goal.hypotheses[*i] else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: *p.clone(),
                }]
            }

            // `P → Q ⊢ R` を `⊢ P` と `Q ⊢ R` に分割
            ForwardTo { i } => {
                let To(p, q) = &goal.hypotheses[*i] else {
                    unreachable!()
                };
                vec![
                    Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: *p.clone(),
                    },
                    Goal {
                        hypotheses: {
                            let mut h = goal.hypotheses.clone();
                            h.push(*q.clone());
                            h
                        },
                        target: goal.target.clone(),
                    },
                ]
            }

            // 仮説 `P ↔ Q` により、結論または仮説の `P`, `Q` を書き換える
            RewriteIff {
                i,
                direction,
                location,
            } => {
                let Iff(p, q) = &goal.hypotheses[*i] else {
                    unreachable!()
                };
                let fml = match direction {
                    RewriteDirection::Fwd => *q.clone(),
                    RewriteDirection::Rev => *p.clone(),
                };
                let mut goal = goal.clone();
                match location {
                    RewriteLocation::Target => goal.target = fml,
                    RewriteLocation::Hypothesis { i } => goal.hypotheses[*i] = fml,
                }
                vec![goal]
            }

            // 仮説 `a = b` により、結論または仮説の `a` を `b` に書き換える
            Rw {
                i,
                direction,
                location,
            } => {
                let Eq(s, t) = &goal.hypotheses[*i] else {
                    unreachable!()
                };
                let (from, to) = match direction {
                    RewriteDirection::Fwd => (s, t),
                    RewriteDirection::Rev => (t, s),
                };
                let mut goal = goal.clone();
                let fml = match location {
                    RewriteLocation::Target => &mut goal.target,
                    RewriteLocation::Hypothesis { i } => &mut goal.hypotheses[*i],
                };
                fml.rewrite(from, to);
                vec![goal]
            }

            // `⊢ s = u` を `⊢ s = t` と `⊢ t = u` に分割
            Calc { t } => {
                let Eq(s, u) = &goal.target else {
                    unreachable!()
                };
                vec![
                    Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: Eq(s.clone(), t.clone()),
                    },
                    Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: Eq(t.clone(), u.clone()),
                    },
                ]
            }

            // `⊢ f(s, t) = f(u, v)` を `⊢ s = u` と `⊢ t = v` に分割
            Congr => {
                let Eq(Fn(_, ss), Fn(_, ts)) = &goal.target else {
                    unreachable!()
                };
                ss.iter()
                    .zip(ts)
                    .filter(|(s, t)| s != t)
                    .map(|(s, t)| Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: Eq(s.clone(), t.clone()),
                    })
                    .collect()
            }

            // `P ∧ Q ⊢` を `P, Q ⊢` に分解
            CasesAnd { i } => {
                let And(p, q) = &goal.hypotheses[*i] else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.remove(*i);
                        h.push(*p.clone());
                        h.push(*q.clone());
                        h
                    },
                    target: goal.target.clone(),
                }]
            }

            // `P ∨ Q ⊢` を `P ⊢` と `Q ⊢` に場合分け
            CasesOr { i } => {
                let Some(Or(p, q)) = goal.hypotheses.get(*i) else {
                    unreachable!()
                };
                vec![
                    Goal {
                        hypotheses: {
                            let mut h = goal.hypotheses.clone();
                            h.remove(*i);
                            h.push(*p.clone());
                            h
                        },
                        target: goal.target.clone(),
                    },
                    Goal {
                        hypotheses: {
                            let mut h = goal.hypotheses.clone();
                            h.remove(*i);
                            h.push(*q.clone());
                            h
                        },
                        target: goal.target.clone(),
                    },
                ]
            }

            // `P ↔ Q ⊢` を `P → Q, Q → P ⊢` に分解
            CasesIff { i } => {
                let Some(Iff(p, q)) = goal.hypotheses.get(*i) else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.remove(*i);
                        h.push(To(p.clone(), q.clone()));
                        h.push(To(q.clone(), p.clone()));
                        h
                    },
                    target: goal.target.clone(),
                }]
            }

            // `∃x P(x) ⊢` を `P(x) ⊢` に変換
            CasesEx { i } => {
                let Ex { v, body, .. } = &goal.hypotheses[*i] else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        let mut used = hashset!();
                        for h in &goal.hypotheses {
                            h.ids(&mut used);
                        }
                        goal.target.ids(&mut used);
                        let v = fresh(v, &used);
                        let mut body = *body.clone();
                        body.open(&Var(v));
                        h.remove(*i);
                        h.push(body);
                        h
                    },
                    target: goal.target.clone(),
                }]
            }

            // `∀x P(x) ⊢` を `∀x P(x), P(t) ⊢` に変換
            SpecializeAll { i, t } => {
                let All { body, .. } = &goal.hypotheses[*i] else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        let mut p = *body.clone();
                        p.open(t);
                        h.push(p);
                        h
                    },
                    target: goal.target.clone(),
                }]
            }

            // `P → Q, P ⊢` を `P → Q, P, Q ⊢` に変換
            SpecializeTo { i } => {
                let To(_, q) = &goal.hypotheses[*i] else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.push(*q.clone());
                        h
                    },
                    target: goal.target.clone(),
                }]
            }

            // 中間命題 `P` を導入し、`⊢ Q` を `⊢ P` と `P ⊢ Q` に分割
            Have { fml } => {
                vec![
                    Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: fml.clone(),
                    },
                    Goal {
                        hypotheses: {
                            let mut h = goal.hypotheses.clone();
                            h.push(fml.clone());
                            h
                        },
                        target: goal.target.clone(),
                    },
                ]
            }

            // 定理を仮説に追加
            UseTheorem { fml } => {
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.push(fml.clone());
                        h
                    },
                    target: goal.target.clone(),
                }]
            }
        }
    }

    /// タクティクが適用可能かを返す。
    /// `i` で仮説を指定する `Tactic` において、すべての `i` が
    /// `goal.hypotheses` の有効な添字であることを呼び出し側で保証すること。
    pub fn can_apply(&self, goal: &Goal) -> bool {
        match self {
            // `⊢ ¬P` を `P ⊢ ⊥` に変換
            IntroNot => matches!(goal.target, Not(_)),

            // `⊢ P → Q` を `P ⊢ Q` に変換
            IntroTo => matches!(goal.target, To(..)),

            // `⊢ ∀x P(x)` を `⊢ P(x)` に変換
            IntroAll => matches!(goal.target, All { .. }),

            // `⊢ P ∧ Q` を `⊢ P` と `⊢ Q` に分割
            ConstructorAnd => matches!(goal.target, And(..)),

            // `⊢ P ↔ Q` を `P ⊢ Q` と `Q ⊢ P` に分割
            ConstructorIff => matches!(goal.target, Iff(..)),

            // `⊢ P ∨ Q` を `⊢ P` に変換
            // `⊢ P ∨ Q` を `⊢ Q` に変換
            Left | Right => matches!(goal.target, Or(..)),

            // `⊢ ∃x P(x)` を `⊢ P(t)` に変換
            Exists { .. } => matches!(goal.target, Ex { .. }),

            // 結論を `⊥` に変更
            // `⊢ P` を `¬P ⊢ ⊥` に変換（背理法）
            Exfalso | ByContra => goal.target != False,

            // 結論または仮説の否定を次の規則で内側へ変形
            // - `¬¬P` を `P` に変換
            // - `¬(P ∨ Q)` を `¬P ∧ ¬Q` に変換
            // - `¬(P ∧ Q)` を `¬P ∨ ¬Q` に変換
            // - `¬(P → Q)` を `P ∧ ¬Q` に変換
            // - `¬∃x P(x)` を `∀x ¬P(x)` に変換
            // - `¬∀x P(x)` を `∃x ¬P(x)` に変換
            PushNot { location } => {
                let np = match location {
                    RewriteLocation::Target => &goal.target,
                    RewriteLocation::Hypothesis { i } => &goal.hypotheses[*i],
                };
                let Not(p) = np else {
                    return false;
                };
                matches!(
                    **p,
                    Not(_) | Or(..) | And(..) | To(..) | Ex { .. } | All { .. }
                )
            }

            // 仮説と結論の一致、`⊥` 仮説、矛盾する仮説から証明完了
            Close => goal.hypotheses.iter().any(|p| {
                p == &goal.target
                    || *p == False
                    || matches!(p, Not(q) if goal.hypotheses.contains(q))
            }),

            // `¬P ⊢ ⊥` を `¬P ⊢ P` に変換
            ApplyNot { i } => goal.target == False && matches!(&goal.hypotheses[*i], Not(_)),

            // `P → Q ⊢ Q` を `P → Q ⊢ P` に変換
            ApplyTo { i } => matches!(&goal.hypotheses[*i], To(_, q) if goal.target == **q),

            // `P → Q ⊢ R` を `⊢ P` と `Q ⊢ R` に分割
            ForwardTo { i } => matches!(&goal.hypotheses[*i], To(_, q) if goal.target != **q),

            // 仮説 `P ↔ Q` により、結論または仮説の `P`, `Q` を書き換える
            RewriteIff {
                i,
                direction,
                location,
            } => {
                let Iff(p, q) = &goal.hypotheses[*i] else {
                    return false;
                };
                let fml = match location {
                    RewriteLocation::Target => &goal.target,
                    RewriteLocation::Hypothesis { i } => &goal.hypotheses[*i],
                };
                match direction {
                    RewriteDirection::Fwd => *fml == **p,
                    RewriteDirection::Rev => *fml == **q,
                }
            }

            // `⊢ t = t` を証明完了
            Rfl => matches!(&goal.target, Eq(s, t) if s == t),

            // 仮説 `a = b` により、結論または仮説の `a` を `b` に書き換える
            Rw {
                i,
                direction,
                location,
            } => {
                let Eq(s, t) = &goal.hypotheses[*i] else {
                    return false;
                };
                let (from, to) = match direction {
                    RewriteDirection::Fwd => (s, t),
                    RewriteDirection::Rev => (t, s),
                };
                if from == to {
                    return false;
                }
                let fml = match location {
                    RewriteLocation::Target => goal.target.clone(),
                    RewriteLocation::Hypothesis { i } => goal.hypotheses[*i].clone(),
                };
                fml.contains(from)
            }

            // `⊢ s = u` を `⊢ s = t` と `⊢ t = u` に分割
            Calc { .. } => matches!(&goal.target, Eq(..)),

            // `⊢ f(s, t) = f(u, v)` を `⊢ s = u` と `⊢ t = v` に分割
            Congr => matches!(
                &goal.target,
                Eq(Fn(f, ss), Fn(g, ts)) if f == g && ss.len() == ts.len() && ss != ts
            ),

            // `P ∧ Q ⊢` を `P, Q ⊢` に分解
            CasesAnd { i } => matches!(&goal.hypotheses[*i], And(..)),

            // `P ∨ Q ⊢` を `P ⊢` と `Q ⊢` に場合分け
            CasesOr { i } => matches!(&goal.hypotheses[*i], Or(..)),

            // `P ↔ Q ⊢` を `P → Q, Q → P ⊢` に分解
            CasesIff { i } => matches!(&goal.hypotheses[*i], Iff(..)),

            // `∃x P(x) ⊢` を `P(x) ⊢` に変換
            CasesEx { i } => matches!(&goal.hypotheses[*i], Ex { .. }),

            // `∀x P(x) ⊢` を `∀x P(x), P(t) ⊢` に変換
            SpecializeAll { i, .. } => matches!(&goal.hypotheses[*i], All { .. }),

            // `P → Q, P ⊢` を `P → Q, P, Q ⊢` に変換
            SpecializeTo { i } => {
                let To(p, _) = &goal.hypotheses[*i] else {
                    return false;
                };
                goal.hypotheses.iter().any(|h| *h == **p)
            }

            // 指定した仮説を削除
            // `⊢ P` を `Q ⊢ P` と `¬Q ⊢ P` に場合分け
            // 中間命題 `P` を導入し、`⊢ Q` を `⊢ P` と `P ⊢ Q` に分割
            // 定理を仮説に追加
            Clear { .. } | ByCases { .. } | Have { .. } | UseTheorem { .. } => true,
        }
    }

    /// タクティクカードに表示する短い名前を返す
    pub const fn label(&self) -> &'static str {
        unimplemented!()
    }

    /// タクティクカードに表示する説明を返す
    pub const fn description(&self) -> &'static str {
        unimplemented!()
    }

    /// タクティク適用前のゴール表示を返す
    pub fn before(&self, _goal: &Goal) -> String {
        unimplemented!()
    }

    /// タクティク適用後のゴール表示を返す
    pub fn after(&self, _goal: &Goal) -> String {
        unimplemented!()
    }
}

impl Term {
    /// 指定した項が含まれるかを返す
    fn contains(&self, t: &Self) -> bool {
        if self == t {
            return true;
        }
        let Fn(_, args) = self else {
            return false;
        };
        args.iter().any(|u| u.contains(t))
    }

    /// `from` を `to` に書き換える
    fn rewrite(&mut self, from: &Self, to: &Self) {
        if self == from {
            *self = to.clone();
            return;
        }
        let Fn(_, args) = self else {
            return;
        };
        for t in args {
            t.rewrite(from, to);
        }
    }
}

impl Formula {
    /// 指定した項が含まれるかを返す
    fn contains(&self, t: &Term) -> bool {
        match self {
            False => false,
            Atom(_, args) => args.iter().any(|u| u.contains(t)),
            Eq(s, t) => s.contains(t) || t.contains(t),
            Not(p) => p.contains(t),
            And(p, q) | Or(p, q) | To(p, q) | Iff(p, q) => p.contains(t) || q.contains(t),
            All { body, .. } | Ex { body, .. } => body.contains(t),
        }
    }

    /// `from` を `to` に書き換える
    fn rewrite(&mut self, from: &Term, to: &Term) {
        match self {
            False => {}
            Atom(_, args) => {
                for t in args {
                    t.rewrite(from, to);
                }
            }
            Eq(s, t) => {
                s.rewrite(from, to);
                t.rewrite(from, to);
            }
            Not(p) => p.rewrite(from, to),
            And(p, q) | Or(p, q) | To(p, q) | Iff(p, q) => {
                p.rewrite(from, to);
                q.rewrite(from, to);
            }
            All { body, .. } | Ex { body, .. } => body.rewrite(from, to),
        }
    }
}
