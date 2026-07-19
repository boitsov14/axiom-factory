use crate::{
    ids::fresh,
    syntax::{Formula, Formula::*, Goal, Id, Term},
};
use Tactic::*;
use maplit::hashset;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theorem {
    pub id: String,
    pub fml: Formula,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RewriteDirection {
    Fwd,
    Rev,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RewriteLocation {
    Target,
    Hypothesis { i: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// `⊢ G` を `P ⊢ G` と `¬P ⊢ G` に場合分け
    ByCases { fml: Formula },
    /// 仮説との一致、`⊥` 仮説、または矛盾する仮説から証明完了
    Close,
    /// `¬P ⊢ ⊥` を `¬P ⊢ P` に変換
    ApplyNot { i: usize },
    /// `P → Q ⊢ Q` を `P → Q ⊢ P` に変換
    ApplyTo { i: usize },
    /// `P → Q ⊢ G` を `⊢ P` と `Q ⊢ G` に分割
    ForwardTo { i: usize },
    /// 仮説 `P ↔ Q` により、結論または仮説中の `P`/`Q` を書き換える
    RewriteIff {
        i: usize,
        direction: RewriteDirection,
        location: RewriteLocation,
    },
    /// `P ∧ Q ⊢` を `P, Q ⊢` に分解
    CasesAnd { i: usize },
    /// `P ∨ Q ⊢` を `P ⊢` と `Q ⊢` に場合分け
    CasesOr { i: usize },
    /// `P ↔ Q ⊢` を `P → Q, Q → P ⊢` に分解
    CasesIff { i: usize },
    /// `∃x P(x) ⊢` を `P(x) ⊢` に変換
    CasesEx { i: usize },
    /// `∀x P(x) ⊢` に `Term t` を代入し `∀x P(x), P(t) ⊢` に変換
    SpecializeAll { i: usize, t: Term },
    /// `P → Q, P ⊢` を `P → Q, P, Q ⊢` に変換
    SpecializeTo { i: usize },
    /// 中間命題 `P` を導入し、その証明と利用のサブゴールを作成
    Have { fml: Formula },
    /// 名前付き定理を仮説に追加
    UseTheorem { id: String },
    /// 名前付き定理 `P ↔ Q` により、結論または仮説を書き換える
    RwIffThm {
        id: String,
        direction: RewriteDirection,
        location: RewriteLocation,
    },
    /// 名前付き定理 `P → Q` により、結論 `Q` を `P` に変換
    ApplyToThm { id: String },
    /// 名前付き定理 `P → Q` と仮説 `P` から、仮説 `Q` を追加
    SpecializeToThm { id: String, i: usize },
}

impl Tactic {
    /// `Goal` にタクティクを適用し、新しいゴールのリストを返す。
    /// `can_apply` で事前に適用可能性を確認すること。
    /// 名前付き定理を使うタクティクには `apply_with_theorems` を使うこと。
    pub fn apply(&self, goal: &Goal) -> Vec<Goal> {
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
                        body.open(&Term::Var(v));
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

            // `⊢ G` を `P ⊢ G` と `¬P ⊢ G` に場合分け
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

            // 仮説との一致、`⊥` 仮説、または矛盾する仮説から証明完了
            Close => {
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

            // `P → Q ⊢ G` を `⊢ P` と `Q ⊢ G` に分割
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

            // 仮説 `P ↔ Q` により、結論または仮説中の `P`/`Q` を書き換える
            RewriteIff {
                i,
                direction,
                location,
            } => {
                let Iff(p, q) = &goal.hypotheses[*i] else {
                    unreachable!()
                };
                let (from, to) = match direction {
                    RewriteDirection::Fwd => (p.as_ref(), q.as_ref()),
                    RewriteDirection::Rev => (q.as_ref(), p.as_ref()),
                };
                rewrite_goal_exact(goal, location, from, to).unwrap_or_else(|| unreachable!())
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
                        body.open(&Term::Var(v));
                        h.remove(*i);
                        h.push(body);
                        h
                    },
                    target: goal.target.clone(),
                }]
            }

            // `∀x P(x) ⊢` に `Term t` を代入し `∀x P(x), P(t) ⊢` に変換
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

            // 中間命題 `P` を導入し、その証明と利用のサブゴールを作成
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

            // 名前付き定理を使うタクティクは `apply_with_theorems` に委譲
            UseTheorem { .. } | RwIffThm { .. } | ApplyToThm { .. } | SpecializeToThm { .. } => {
                panic!("this tactic requires apply_with_theorems")
            }
        }
    }

    /// 名前付き定理を参照できる状態で、`Goal` にタクティクを適用し、新しいゴールのリストを返す。
    /// `can_apply_with_theorems` で事前に適用可能性を確認すること。
    pub fn apply_with_theorems(&self, goal: &Goal, theorems: &[Theorem]) -> Vec<Goal> {
        match self {
            // 名前付き定理を仮説に追加
            UseTheorem { id } => {
                let thm = find_theorem(theorems, id).unwrap_or_else(|| unreachable!());
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.push(thm.fml.clone());
                        h
                    },
                    target: goal.target.clone(),
                }]
            }

            // 名前付き定理 `P ↔ Q` により、結論または仮説を書き換える
            RwIffThm {
                id,
                direction,
                location,
            } => {
                let thm = find_theorem(theorems, id).unwrap_or_else(|| unreachable!());
                let Iff(p, q) = &thm.fml else { unreachable!() };
                let (pattern, replacement) = match direction {
                    RewriteDirection::Fwd => (p.as_ref(), q.as_ref()),
                    RewriteDirection::Rev => (q.as_ref(), p.as_ref()),
                };
                rewrite_goal_schema(goal, location, pattern, replacement)
                    .unwrap_or_else(|| unreachable!())
            }

            // 名前付き定理 `P → Q` により、結論 `Q` を `P` に変換
            ApplyToThm { id } => {
                let thm = find_theorem(theorems, id).unwrap_or_else(|| unreachable!());
                let To(p, q) = &thm.fml else { unreachable!() };
                let mut subst = SchemaSubst::default();
                if !match_formula_pattern(q, &goal.target, &mut subst)
                    || !all_schema_atoms_bound(p, &subst)
                {
                    unreachable!()
                }
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: instantiate_schema(p, &subst),
                }]
            }

            // 名前付き定理 `P → Q` と仮説 `P` から、仮説 `Q` を追加
            SpecializeToThm { id, i } => {
                let thm = find_theorem(theorems, id).unwrap_or_else(|| unreachable!());
                let To(p, q) = &thm.fml else { unreachable!() };
                let hyp = goal.hypotheses.get(*i).unwrap_or_else(|| unreachable!());
                let mut subst = SchemaSubst::default();
                if !match_formula_pattern(p, hyp, &mut subst) || !all_schema_atoms_bound(q, &subst)
                {
                    unreachable!()
                }
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.push(instantiate_schema(q, &subst));
                        h
                    },
                    target: goal.target.clone(),
                }]
            }

            // 名前付き定理を使わないタクティクは通常の `apply` に委譲
            _ => self.apply(goal),
        }
    }

    /// タクティクが適用可能かを返す。
    /// 名前付き定理を使うタクティクには `can_apply_with_theorems` を使うこと。
    pub fn can_apply(&self, goal: &Goal) -> bool {
        match self {
            IntroNot => matches!(goal.target, Not(_)),
            IntroTo => matches!(goal.target, To(..)),
            IntroAll => matches!(goal.target, All { .. }),
            ConstructorAnd => matches!(goal.target, And(..)),
            ConstructorIff => matches!(goal.target, Iff(..)),
            Left => matches!(goal.target, Or(..)),
            Right => matches!(goal.target, Or(..)),
            Exists { .. } => matches!(goal.target, Ex { .. }),
            Exfalso => goal.target != False,
            ByContra => goal.target != False,
            ByCases { .. } => true,
            Close => can_close(goal),
            ApplyNot { i } => {
                goal.target == False && goal.hypotheses.get(*i).is_some_and(|h| matches!(h, Not(_)))
            }
            ApplyTo { i } => goal
                .hypotheses
                .get(*i)
                .is_some_and(|h| matches!(h, To(_, q) if q.as_ref() == &goal.target)),
            ForwardTo { i } => goal.hypotheses.get(*i).is_some_and(|h| matches!(h, To(..))),
            RewriteIff {
                i,
                direction,
                location,
            } => {
                let Some(Iff(p, q)) = goal.hypotheses.get(*i) else {
                    return false;
                };
                let (from, to) = match direction {
                    RewriteDirection::Fwd => (p.as_ref(), q.as_ref()),
                    RewriteDirection::Rev => (q.as_ref(), p.as_ref()),
                };
                rewrite_goal_exact(goal, location, from, to).is_some()
            }
            CasesAnd { i } => goal
                .hypotheses
                .get(*i)
                .is_some_and(|h| matches!(h, And(..))),
            CasesOr { i } => goal.hypotheses.get(*i).is_some_and(|h| matches!(h, Or(..))),
            CasesIff { i } => goal
                .hypotheses
                .get(*i)
                .is_some_and(|h| matches!(h, Iff(..))),
            CasesEx { i } => goal
                .hypotheses
                .get(*i)
                .is_some_and(|h| matches!(h, Ex { .. })),
            SpecializeAll { i, .. } => goal
                .hypotheses
                .get(*i)
                .is_some_and(|h| matches!(h, All { .. })),
            SpecializeTo { i } => {
                let Some(To(p, _)) = goal.hypotheses.get(*i) else {
                    return false;
                };
                goal.hypotheses.iter().any(|h| h == p.as_ref())
            }
            Have { .. } => true,
            UseTheorem { .. } | RwIffThm { .. } | ApplyToThm { .. } | SpecializeToThm { .. } => {
                false
            }
        }
    }

    /// 名前付き定理を参照できる状態で、タクティクが適用可能かを返す。
    pub fn can_apply_with_theorems(&self, goal: &Goal, theorems: &[Theorem]) -> bool {
        match self {
            UseTheorem { id } => find_theorem(theorems, id).is_some(),
            RwIffThm {
                id,
                direction,
                location,
            } => {
                let Some(thm) = find_theorem(theorems, id) else {
                    return false;
                };
                let Iff(p, q) = &thm.fml else {
                    return false;
                };
                let (pattern, replacement) = match direction {
                    RewriteDirection::Fwd => (p.as_ref(), q.as_ref()),
                    RewriteDirection::Rev => (q.as_ref(), p.as_ref()),
                };
                rewrite_goal_schema(goal, location, pattern, replacement).is_some()
            }
            ApplyToThm { id } => {
                let Some(thm) = find_theorem(theorems, id) else {
                    return false;
                };
                let To(p, q) = &thm.fml else {
                    return false;
                };
                let mut subst = SchemaSubst::default();
                match_formula_pattern(q, &goal.target, &mut subst)
                    && all_schema_atoms_bound(p, &subst)
            }
            SpecializeToThm { id, i } => {
                let Some(thm) = find_theorem(theorems, id) else {
                    return false;
                };
                let To(p, q) = &thm.fml else {
                    return false;
                };
                let Some(hyp) = goal.hypotheses.get(*i) else {
                    return false;
                };
                let mut subst = SchemaSubst::default();
                match_formula_pattern(p, hyp, &mut subst) && all_schema_atoms_bound(q, &subst)
            }
            _ => self.can_apply(goal),
        }
    }

    pub const fn label(&self) -> &'static str {
        unimplemented!()
    }

    pub const fn description(&self) -> &'static str {
        unimplemented!()
    }

    pub fn before(&self, _goal: &Goal) -> String {
        unimplemented!()
    }

    pub fn after(&self, _goal: &Goal) -> String {
        unimplemented!()
    }
}

fn find_theorem<'a>(theorems: &'a [Theorem], id: &str) -> Option<&'a Theorem> {
    theorems.iter().find(|thm| thm.id == id)
}

fn can_close(goal: &Goal) -> bool {
    goal.hypotheses.iter().any(|h| h == &goal.target)
        || goal.hypotheses.iter().any(|h| matches!(h, False))
        || has_contradiction(&goal.hypotheses)
}

fn has_contradiction(hypotheses: &[Formula]) -> bool {
    hypotheses.iter().any(|h| match h {
        Not(p) => hypotheses.iter().any(|g| g == p.as_ref()),
        _ => hypotheses
            .iter()
            .any(|g| matches!(g, Not(p) if p.as_ref() == h)),
    })
}

fn rewrite_goal_exact(
    goal: &Goal,
    location: &RewriteLocation,
    from: &Formula,
    to: &Formula,
) -> Option<Vec<Goal>> {
    match location {
        RewriteLocation::Target => rewrite_formula_exact(&goal.target, from, to).map(|target| {
            vec![Goal {
                hypotheses: goal.hypotheses.clone(),
                target,
            }]
        }),
        RewriteLocation::Hypothesis { i } => {
            let hyp = goal.hypotheses.get(*i)?;
            rewrite_formula_exact(hyp, from, to).map(|new_hyp| {
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h[*i] = new_hyp;
                        h
                    },
                    target: goal.target.clone(),
                }]
            })
        }
    }
}

fn rewrite_formula_exact(fml: &Formula, from: &Formula, to: &Formula) -> Option<Formula> {
    if fml == from {
        return Some(to.clone());
    }

    match fml {
        False | Atom(..) | Eq(..) => None,
        Not(p) => rewrite_formula_exact(p, from, to).map(|p| Not(Box::new(p))),
        And(p, q) => rewrite_binary_exact(p, q, from, to, And),
        Or(p, q) => rewrite_binary_exact(p, q, from, to, Or),
        To(p, q) => rewrite_binary_exact(p, q, from, to, To),
        Iff(p, q) => rewrite_binary_exact(p, q, from, to, Iff),
        All { v, sort, body } => rewrite_formula_exact(body, from, to).map(|body| All {
            v: v.clone(),
            sort: sort.clone(),
            body: Box::new(body),
        }),
        Ex { v, sort, body } => rewrite_formula_exact(body, from, to).map(|body| Ex {
            v: v.clone(),
            sort: sort.clone(),
            body: Box::new(body),
        }),
    }
}

fn rewrite_binary_exact(
    p: &Formula,
    q: &Formula,
    from: &Formula,
    to: &Formula,
    ctor: fn(Box<Formula>, Box<Formula>) -> Formula,
) -> Option<Formula> {
    let p2 = rewrite_formula_exact(p, from, to);
    let q2 = rewrite_formula_exact(q, from, to);

    if p2.is_none() && q2.is_none() {
        None
    } else {
        Some(ctor(
            Box::new(p2.unwrap_or_else(|| p.clone())),
            Box::new(q2.unwrap_or_else(|| q.clone())),
        ))
    }
}

fn rewrite_goal_schema(
    goal: &Goal,
    location: &RewriteLocation,
    pattern: &Formula,
    replacement: &Formula,
) -> Option<Vec<Goal>> {
    match location {
        RewriteLocation::Target => {
            rewrite_formula_schema(&goal.target, pattern, replacement).map(|target| {
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target,
                }]
            })
        }
        RewriteLocation::Hypothesis { i } => {
            let hyp = goal.hypotheses.get(*i)?;
            rewrite_formula_schema(hyp, pattern, replacement).map(|new_hyp| {
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h[*i] = new_hyp;
                        h
                    },
                    target: goal.target.clone(),
                }]
            })
        }
    }
}

fn rewrite_formula_schema(
    fml: &Formula,
    pattern: &Formula,
    replacement: &Formula,
) -> Option<Formula> {
    let mut subst = SchemaSubst::default();
    if match_formula_pattern(pattern, fml, &mut subst)
        && all_schema_atoms_bound(replacement, &subst)
    {
        return Some(instantiate_schema(replacement, &subst));
    }

    match fml {
        False | Atom(..) | Eq(..) => None,
        Not(p) => rewrite_formula_schema(p, pattern, replacement).map(|p| Not(Box::new(p))),
        And(p, q) => rewrite_binary_schema(p, q, pattern, replacement, And),
        Or(p, q) => rewrite_binary_schema(p, q, pattern, replacement, Or),
        To(p, q) => rewrite_binary_schema(p, q, pattern, replacement, To),
        Iff(p, q) => rewrite_binary_schema(p, q, pattern, replacement, Iff),
        All { v, sort, body } => {
            rewrite_formula_schema(body, pattern, replacement).map(|body| All {
                v: v.clone(),
                sort: sort.clone(),
                body: Box::new(body),
            })
        }
        Ex { v, sort, body } => rewrite_formula_schema(body, pattern, replacement).map(|body| Ex {
            v: v.clone(),
            sort: sort.clone(),
            body: Box::new(body),
        }),
    }
}

fn rewrite_binary_schema(
    p: &Formula,
    q: &Formula,
    pattern: &Formula,
    replacement: &Formula,
    ctor: fn(Box<Formula>, Box<Formula>) -> Formula,
) -> Option<Formula> {
    let p2 = rewrite_formula_schema(p, pattern, replacement);
    let q2 = rewrite_formula_schema(q, pattern, replacement);

    if p2.is_none() && q2.is_none() {
        None
    } else {
        Some(ctor(
            Box::new(p2.unwrap_or_else(|| p.clone())),
            Box::new(q2.unwrap_or_else(|| q.clone())),
        ))
    }
}

#[derive(Clone, Debug, Default)]
struct SchemaSubst {
    bindings: HashMap<Id, SchemaBinding>,
}

#[derive(Clone, Debug)]
struct SchemaBinding {
    params: Vec<Term>,
    body: Formula,
}

fn match_formula_pattern(pattern: &Formula, target: &Formula, subst: &mut SchemaSubst) -> bool {
    match pattern {
        False => matches!(target, False),
        Atom(id, args) => bind_schema_atom(id, args, target, subst),
        Eq(l, r) => {
            let Eq(l2, r2) = target else {
                return false;
            };
            l == l2 && r == r2
        }
        Not(p) => {
            let Not(p2) = target else {
                return false;
            };
            match_formula_pattern(p, p2, subst)
        }
        And(p, q) => {
            let And(p2, q2) = target else {
                return false;
            };
            match_formula_pattern(p, p2, subst) && match_formula_pattern(q, q2, subst)
        }
        Or(p, q) => {
            let Or(p2, q2) = target else {
                return false;
            };
            match_formula_pattern(p, p2, subst) && match_formula_pattern(q, q2, subst)
        }
        To(p, q) => {
            let To(p2, q2) = target else {
                return false;
            };
            match_formula_pattern(p, p2, subst) && match_formula_pattern(q, q2, subst)
        }
        Iff(p, q) => {
            let Iff(p2, q2) = target else {
                return false;
            };
            match_formula_pattern(p, p2, subst) && match_formula_pattern(q, q2, subst)
        }
        All { sort, body, .. } => {
            let All {
                sort: sort2,
                body: body2,
                ..
            } = target
            else {
                return false;
            };
            sort == sort2 && match_formula_pattern(body, body2, subst)
        }
        Ex { sort, body, .. } => {
            let Ex {
                sort: sort2,
                body: body2,
                ..
            } = target
            else {
                return false;
            };
            sort == sort2 && match_formula_pattern(body, body2, subst)
        }
    }
}

fn bind_schema_atom(id: &Id, args: &[Term], target: &Formula, subst: &mut SchemaSubst) -> bool {
    if let Some(binding) = subst.bindings.get(id) {
        if binding.params.len() != args.len() {
            return false;
        }
        instantiate_binding(binding, args) == *target
    } else {
        subst.bindings.insert(
            id.clone(),
            SchemaBinding {
                params: args.to_vec(),
                body: target.clone(),
            },
        );
        true
    }
}

fn instantiate_schema(fml: &Formula, subst: &SchemaSubst) -> Formula {
    match fml {
        False => False,
        Atom(id, args) => subst
            .bindings
            .get(id).map_or_else(|| Atom(id.clone(), args.clone()), |binding| instantiate_binding(binding, args)),
        Eq(l, r) => Eq(l.clone(), r.clone()),
        Not(p) => Not(Box::new(instantiate_schema(p, subst))),
        And(p, q) => And(
            Box::new(instantiate_schema(p, subst)),
            Box::new(instantiate_schema(q, subst)),
        ),
        Or(p, q) => Or(
            Box::new(instantiate_schema(p, subst)),
            Box::new(instantiate_schema(q, subst)),
        ),
        To(p, q) => To(
            Box::new(instantiate_schema(p, subst)),
            Box::new(instantiate_schema(q, subst)),
        ),
        Iff(p, q) => Iff(
            Box::new(instantiate_schema(p, subst)),
            Box::new(instantiate_schema(q, subst)),
        ),
        All { v, sort, body } => All {
            v: v.clone(),
            sort: sort.clone(),
            body: Box::new(instantiate_schema(body, subst)),
        },
        Ex { v, sort, body } => Ex {
            v: v.clone(),
            sort: sort.clone(),
            body: Box::new(instantiate_schema(body, subst)),
        },
    }
}

fn instantiate_binding(binding: &SchemaBinding, args: &[Term]) -> Formula {
    substitute_terms_in_formula(&binding.body, &binding.params, args)
}

fn substitute_terms_in_formula(fml: &Formula, from: &[Term], to: &[Term]) -> Formula {
    match fml {
        False => False,
        Atom(id, args) => Atom(
            id.clone(),
            args.iter()
                .map(|arg| substitute_terms_in_term(arg, from, to))
                .collect(),
        ),
        Eq(l, r) => Eq(
            substitute_terms_in_term(l, from, to),
            substitute_terms_in_term(r, from, to),
        ),
        Not(p) => Not(Box::new(substitute_terms_in_formula(p, from, to))),
        And(p, q) => And(
            Box::new(substitute_terms_in_formula(p, from, to)),
            Box::new(substitute_terms_in_formula(q, from, to)),
        ),
        Or(p, q) => Or(
            Box::new(substitute_terms_in_formula(p, from, to)),
            Box::new(substitute_terms_in_formula(q, from, to)),
        ),
        To(p, q) => To(
            Box::new(substitute_terms_in_formula(p, from, to)),
            Box::new(substitute_terms_in_formula(q, from, to)),
        ),
        Iff(p, q) => Iff(
            Box::new(substitute_terms_in_formula(p, from, to)),
            Box::new(substitute_terms_in_formula(q, from, to)),
        ),
        All { v, sort, body } => All {
            v: v.clone(),
            sort: sort.clone(),
            body: Box::new(substitute_terms_in_formula(body, from, to)),
        },
        Ex { v, sort, body } => Ex {
            v: v.clone(),
            sort: sort.clone(),
            body: Box::new(substitute_terms_in_formula(body, from, to)),
        },
    }
}

fn substitute_terms_in_term(term: &Term, from: &[Term], to: &[Term]) -> Term {
    if let Some((_, replacement)) = from
        .iter()
        .zip(to.iter())
        .find(|(pattern, _)| *pattern == term)
    {
        return replacement.clone();
    }

    match term {
        Term::Var(_) | Term::Bound(_) => term.clone(),
        Term::Fn(id, args) => Term::Fn(
            id.clone(),
            args.iter()
                .map(|arg| substitute_terms_in_term(arg, from, to))
                .collect(),
        ),
    }
}

fn all_schema_atoms_bound(fml: &Formula, subst: &SchemaSubst) -> bool {
    match fml {
        False | Eq(..) => true,
        Atom(id, _) => subst.bindings.contains_key(id),
        Not(p) => all_schema_atoms_bound(p, subst),
        And(p, q) | Or(p, q) | To(p, q) | Iff(p, q) => {
            all_schema_atoms_bound(p, subst) && all_schema_atoms_bound(q, subst)
        }
        All { body, .. } | Ex { body, .. } => all_schema_atoms_bound(body, subst),
    }
}
