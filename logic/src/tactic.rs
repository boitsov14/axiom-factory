use crate::{
    ids::fresh,
    syntax::{Formula, Formula::*, Goal, Term},
};
use maplit::hashset;
use Tactic::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tactic {
    IntroNot,
    IntroTo,
    IntroAll,
    ConstructorAnd,
    ConstructorIff,
    Left,
    Right,
    Exists { t: Term },
    Exfalso,
    ByContra,
    Assumption,
    ApplyNot { i: usize },
    ApplyTo { i: usize },
    ApplyIff { i: usize },
    CasesAnd { i: usize },
    CasesOr { i: usize },
    CasesIff { i: usize },
    CasesEx { i: usize },
    SpecializeAll { i: usize, t: Term },
    SpecializeTo { i: usize },
    Have { fml: Formula },
}

impl Tactic {
    /// `Goal` にタクティクを適用し、新しいゴールのリストを返す。
    /// `can_apply` で事前に適用可能性を確認すること。
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
                let mut used = hashset!();
                for h in &goal.hypotheses {
                    h.ids(&mut used);
                }
                let x = fresh(v, &used);
                let mut p = *body.clone();
                p.open(&Term::Var(x));
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: p,
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
                let mut p = *body.clone();
                p.open(t);
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: p,
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
                let p = goal.target.clone();
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.push(Not(Box::new(p)));
                        h
                    },
                    target: False,
                }]
            }

            // 仮説のうち結論と一致するものがあれば証明完了
            Assumption => {
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

            // `P → Q ⊢` を `⊢ P` と `Q ⊢` に分割
            ApplyTo { i } => {
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

            // `P ↔ Q ⊢ P` を `P ↔ Q ⊢ Q に変換
            // `P ↔ Q ⊢ Q` を `P ↔ Q ⊢ P に変換
            ApplyIff { i } => {
                let Some(Iff(p, q)) = goal.hypotheses.get(*i) else {
                    unreachable!()
                };
                if **p == goal.target {
                    vec![Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: *q.clone(),
                    }]
                } else if **q == goal.target {
                    vec![Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: *p.clone(),
                    }]
                } else {
                    unreachable!()
                }
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
                let mut used = hashset!();
                for h in &goal.hypotheses {
                    h.ids(&mut used);
                }
                goal.target.ids(&mut used);
                let x = fresh(v, &used);
                let mut p = *body.clone();
                p.open(&Term::Var(x));
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
                        h.remove(*i);
                        h.push(p);
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
                let mut p = *body.clone();
                p.open(t);
                vec![Goal {
                    hypotheses: {
                        let mut h = goal.hypotheses.clone();
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
        }
    }

    /// タクティクが適用可能かを返す。
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
            Assumption => goal.hypotheses.iter().any(|h| h == &goal.target),
            ApplyNot { i } => goal.hypotheses.get(*i).is_some_and(|h| matches!(h, Not(_))),
            ApplyTo { i } => goal.hypotheses.get(*i).is_some_and(|h| {
                matches!(h, To(_, q) if q.as_ref() == &goal.target)
            }),
            ApplyIff { i } => goal.hypotheses.get(*i).is_some_and(|h| {
                matches!(h, Iff(p, q) if q.as_ref() == &goal.target || p.as_ref() == &goal.target)
            }),
            CasesAnd { i } => {
                goal.hypotheses.get(*i).is_some_and(|h| matches!(h, And(..)))
            }
            CasesOr { i } => {
                goal.hypotheses.get(*i).is_some_and(|h| matches!(h, Or(..)))
            }
            CasesIff { i } => {
                goal.hypotheses.get(*i).is_some_and(|h| matches!(h, Iff(..)))
            }
            CasesEx { i } => {
                goal.hypotheses.get(*i).is_some_and(|h| matches!(h, Ex { .. }))
            }
            SpecializeAll { i, .. } => {
                goal.hypotheses.get(*i).is_some_and(|h| matches!(h, All { .. }))
            }
            SpecializeTo { i } => {
                let Some(To(p, _)) = goal.hypotheses.get(*i) else {
                    return false;
                };
                goal.hypotheses.iter().any(|h| h == p.as_ref())
            }
            Have { .. } => true,
        }
    }

    /// タクティクの表示名を返す。
    pub const fn label(&self) -> &'static str {
        match self {
            IntroNot => "Intro¬",
            IntroTo => "Intro→",
            IntroAll => "Intro∀",
            ConstructorAnd => "Conj∧",
            ConstructorIff => "Conj↔",
            Left => "Left",
            Right => "Right",
            Exists { .. } => "Exists",
            Exfalso => "ExFalso",
            ByContra => "ByContra",
            Assumption => "Assumption",
            ApplyNot { .. } => "Apply¬",
            ApplyTo { .. } => "Apply→",
            ApplyIff { .. } => "Apply↔",
            CasesAnd { .. } => "Cases∧",
            CasesOr { .. } => "Cases∨",
            CasesIff { .. } => "Cases↔",
            CasesEx { .. } => "Cases∃",
            SpecializeAll { .. } => "Specialize∀",
            SpecializeTo { .. } => "Specialize→",
            Have { .. } => "Have",
        }
    }

    /// タクティクの概要を日本語で返す。
    pub const fn description(&self) -> &'static str {
        match self {
            IntroNot => "結論の否定を仮定に移す",
            IntroTo => "含意の前件を仮定に加える",
            IntroAll => "全称量化子を外して自由変数にする",
            ConstructorAnd => "連言の結論を二つのサブゴールに分割する",
            ConstructorIff => "同値の結論を二方向の含意に分割する",
            Left => "選言の左側を選んで証明する",
            Right => "選言の右側を選んで証明する",
            Exists { .. } => "存在量化の証拠（witness）を与える",
            Exfalso => "結論を⊥に変える（爆発原理）",
            ByContra => "背理法：結論の否定を仮定して⊥を導く",
            Assumption => "仮定のうち結論と一致するもので閉じる",
            ApplyNot { .. } => "否定の仮定を適用し、その否定を結論にする",
            ApplyTo { .. } => "含意の仮定を適用し、前件の証明と後件の利用に分ける",
            ApplyIff { .. } => "同値の仮定を結論に合わせて適用する",
            CasesAnd { .. } => "連言の仮定を二つの仮定に分解する",
            CasesOr { .. } => "選言の仮定を場合分けする",
            CasesIff { .. } => "同値の仮定を二つの含意に分解する",
            CasesEx { .. } => "存在量化の仮定を具体化する",
            SpecializeAll { .. } => "全称仮定を項で具体化する",
            SpecializeTo { .. } => "含意の仮定を前件の仮定を用いて後件を導く",
            Have { .. } => "中間命題を導入し、それを証明してから利用する",
        }
    }

    /// 適用前の状態を表す文字列を返す。
    /// 例えば `IntroTo` なら `"⊢ P → Q"` を返す。
    pub fn before(&self, goal: &Goal) -> String {
        match self {
            IntroNot
            | IntroTo
            | IntroAll
            | ConstructorAnd
            | ConstructorIff
            | Left
            | Right
            | Exists { .. }
            | Exfalso
            | ByContra
            | Have { .. } => format!("⊢ {}", goal.target),

            Assumption => {
                let t = goal.target.to_string();
                format!("{t} ⊢ {t}")
            }

            ApplyNot { i: hypotheses }
            | ApplyTo { i: hypotheses }
            | ApplyIff { i: hypotheses }
            | CasesAnd { i: hypotheses }
            | CasesOr { i: hypotheses }
            | CasesIff { i: hypotheses }
            | CasesEx { i: hypotheses }
            | SpecializeAll { i: hypotheses, .. }
            | SpecializeTo { i: hypotheses, .. } => {
                format!("{} ⊢", goal.hypotheses[*hypotheses])
            }
        }
    }

    /// 適用後の状態を表す文字列を返す。
    /// 複数のサブゴールがある場合は改行で区切る。
    pub fn after(&self, goal: &Goal) -> String {
        match self {
            IntroNot => {
                let Not(p) = &goal.target else { unreachable!() };
                format!("{p} ⊢ ⊥")
            }
            IntroTo => {
                let To(p, q) = &goal.target else {
                    unreachable!()
                };
                format!("{p} ⊢ {q}")
            }
            IntroAll => {
                format!("⊢ {}", goal.target)
            }
            ConstructorAnd => {
                let And(p, q) = &goal.target else {
                    unreachable!()
                };
                format!("⊢ {p}\n⊢ {q}")
            }
            ConstructorIff => {
                let Iff(p, q) = &goal.target else {
                    unreachable!()
                };
                format!("{p} ⊢ {q}\n{q} ⊢ {p}")
            }
            Left => {
                let Or(p, _) = &goal.target else {
                    unreachable!()
                };
                format!("⊢ {p}")
            }
            Right => {
                let Or(_, q) = &goal.target else {
                    unreachable!()
                };
                format!("⊢ {q}")
            }
            Exists { t } => {
                let Ex { body, .. } = &goal.target else {
                    unreachable!()
                };
                let mut p = *body.clone();
                p.open(t);
                format!("⊢ {p}")
            }
            Exfalso => "⊢ ⊥".into(),
            ByContra => {
                let p = goal.target.to_string();
                format!("¬{p} ⊢ ⊥")
            }
            Assumption => String::new(),
            ApplyNot { i: hypotheses } => {
                let Not(p) = &goal.hypotheses[*hypotheses] else {
                    unreachable!()
                };
                format!("⊢ {p}")
            }
            ApplyTo { i: hypotheses } => {
                let To(p, _q) = &goal.hypotheses[*hypotheses] else {
                    unreachable!()
                };
                format!("⊢ {p}\n{_q} ⊢")
            }
            ApplyIff { i: hypotheses } => {
                let Iff(p, q) = &goal.hypotheses[*hypotheses] else {
                    unreachable!()
                };
                if q.as_ref() == &goal.target {
                    format!("⊢ {p}")
                } else {
                    format!("⊢ {q}")
                }
            }
            CasesAnd { i: hypotheses } => {
                let And(p, q) = &goal.hypotheses[*hypotheses] else {
                    unreachable!()
                };
                format!("{p}, {q} ⊢")
            }
            CasesOr { i: hypotheses } => {
                let Or(p, q) = &goal.hypotheses[*hypotheses] else {
                    unreachable!()
                };
                format!("{p} ⊢\n{q} ⊢")
            }
            CasesIff { i: hypotheses } => {
                let Iff(p, q) = &goal.hypotheses[*hypotheses] else {
                    unreachable!()
                };
                format!("{p}, {q} ⊢")
            }
            CasesEx { i: hypotheses } => {
                let Ex { v, body, .. } = &goal.hypotheses[*hypotheses] else {
                    unreachable!()
                };
                let mut p = *body.clone();
                p.open(&Term::Var(v.clone()));
                format!("{p} ⊢")
            }
            SpecializeAll {
                i: hypotheses,
                t: term,
            } => {
                let All { body, .. } = &goal.hypotheses[*hypotheses] else {
                    unreachable!()
                };
                let mut p = *body.clone();
                p.open(term);
                format!("{p} ⊢")
            }
            SpecializeTo { i: hypotheses, .. } => {
                let To(_p, q) = &goal.hypotheses[*hypotheses] else {
                    unreachable!()
                };
                format!("{q} ⊢")
            }
            Have { fml } => {
                format!("⊢ {}\n{} ⊢ {}", fml, fml, goal.target)
            }
        }
    }
}
