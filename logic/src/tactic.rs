use crate::syntax::{Formula, Formula::*, Goal, Term};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tactic {
    IntroNot,
    IntroTo,
    IntroAll,
    ConstructorAnd,
    ConstructorIff,
    Left,
    Right,
    Exists { term: Term },
    Exfalso,
    ByContra,
    Assumption,
    ApplyNot { hyp: usize },
    ApplyTo { hyp: usize },
    ApplyIff { hyp: usize },
    CasesAnd { hyp: usize },
    CasesOr { hyp: usize },
    CasesIff { hyp: usize },
    CasesEx { hyp: usize },
    CasesFalse { hyp: usize },
    SpecializeAll { hyp: usize, term: Term },
    SpecializeTo { hyp: usize, arg_hyp: usize },
    Have { formula: Formula },
}

impl Tactic {
    /// `Goal` にタクティクを適用し、新しいゴールのリストを返す。
    /// `can_apply` で事前に適用可能性を確認すること。
    pub fn apply(&self, goal: &Goal) -> Vec<Goal> {
        match self {
            // ターゲット `⊢ ¬P` を `P ⊢ ⊥` に変換
            Self::IntroNot => {
                let Not(p) = &goal.target else { unreachable!() };
                let mut next = goal.clone();
                next.hypotheses.push(*p.clone());
                next.target = False;
                vec![next]
            }

            // ターゲット `⊢ P → Q` を `P ⊢ Q` に変換
            Self::IntroTo => {
                let To(p, q) = &goal.target else {
                    unreachable!()
                };
                let mut next = goal.clone();
                next.hypotheses.push(*p.clone());
                next.target = *q.clone();
                vec![next]
            }

            // ターゲット `⊢ ∀x P(x)` を `⊢ P(x)` に変換
            Self::IntroAll => {
                let All { v, body, .. } = &goal.target else {
                    unreachable!()
                };
                let mut p = *body.clone();
                p.open(&Term::Var(v.clone()));
                let mut next = goal.clone();
                next.target = p;
                vec![next]
            }

            // ターゲット `⊢ P ∧ Q` を `⊢ P` と `⊢ Q` に分割
            Self::ConstructorAnd => {
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

            // ターゲット `⊢ P ↔ Q` を `P ⊢ Q` と `Q ⊢ P` に分割
            Self::ConstructorIff => {
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

            // ターゲット `⊢ P ∨ Q` から左 `⊢ P` を選択
            Self::Left => {
                let Or(p, _) = &goal.target else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: *p.clone(),
                }]
            }

            // ターゲット `⊢ P ∨ Q` から右 `⊢ Q` を選択
            Self::Right => {
                let Or(_, q) = &goal.target else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: *q.clone(),
                }]
            }

            // ターゲット `⊢ ∃x P(x)` を `⊢ P(t)` に変換
            Self::Exists { term } => {
                let Ex { body, .. } = &goal.target else {
                    unreachable!()
                };
                let mut p = *body.clone();
                p.open(term);
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: p,
                }]
            }

            // ターゲットを `⊥` に変更（爆発原理）
            Self::Exfalso => {
                assert!(goal.target != False, "exfalso: target is already ⊥");
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: False,
                }]
            }

            // ターゲット `⊢ P` を `¬P ⊢ ⊥` に変換（背理法）
            Self::ByContra => {
                assert!(goal.target != False, "by_contra: target is already ⊥");
                let p = goal.target.clone();
                let mut next = goal.clone();
                next.hypotheses.push(Not(Box::new(p)));
                next.target = False;
                vec![next]
            }

            // 仮説のうちゴールと一致するものがあれば証明完了
            Self::Assumption => {
                assert!(
                    goal.hypotheses.iter().any(|h| h == &goal.target),
                    "assumption: no matching hypothesis"
                );
                vec![]
            }

            // 仮定 `¬P ⊢` を `⊢ P` に変換
            Self::ApplyNot { hyp } => {
                let Some(Not(p)) = goal.hypotheses.get(*hyp) else {
                    unreachable!()
                };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: *p.clone(),
                }]
            }

            // 仮定 `P → Q ⊢` を `⊢ P` と `Q ⊢` に変換
            Self::ApplyTo { hyp } => {
                let Some(To(_p, q)) = goal.hypotheses.get(*hyp) else {
                    unreachable!()
                };
                assert!(
                    q.as_ref() == &goal.target,
                    "apply_to: conclusion does not match target"
                );
                let To(p, _) = &goal.hypotheses[*hyp] else {
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

            // 仮定 `P ↔ Q ⊢` をターゲットに合わせて `⊢ P` または `⊢ Q` に変換
            Self::ApplyIff { hyp } => {
                let Some(Iff(p, q)) = goal.hypotheses.get(*hyp) else {
                    unreachable!()
                };
                if q.as_ref() == &goal.target {
                    vec![Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: *p.clone(),
                    }]
                } else if p.as_ref() == &goal.target {
                    vec![Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: *q.clone(),
                    }]
                } else {
                    unreachable!("apply_iff: neither side matches target")
                }
            }

            // 仮定 `P ∧ Q ⊢` を `P, Q ⊢` に分解
            Self::CasesAnd { hyp } => {
                let Some(And(p, q)) = goal.hypotheses.get(*hyp) else {
                    unreachable!()
                };
                let mut next = goal.clone();
                next.hypotheses.remove(*hyp);
                next.hypotheses.push(*p.clone());
                next.hypotheses.push(*q.clone());
                vec![next]
            }

            // 仮定 `P ∨ Q ⊢` を `P ⊢` と `Q ⊢` に場合分け
            Self::CasesOr { hyp } => {
                let Some(Or(p, q)) = goal.hypotheses.get(*hyp) else {
                    unreachable!()
                };
                let mut left = goal.clone();
                left.hypotheses.remove(*hyp);
                left.hypotheses.push(*p.clone());
                let mut right = goal.clone();
                right.hypotheses.remove(*hyp);
                right.hypotheses.push(*q.clone());
                vec![left, right]
            }

            // 仮定 `P ↔ Q ⊢` を `P, Q ⊢` に分解
            Self::CasesIff { hyp } => {
                let Some(Iff(p, q)) = goal.hypotheses.get(*hyp) else {
                    unreachable!()
                };
                let mut next = goal.clone();
                next.hypotheses.remove(*hyp);
                next.hypotheses.push(*p.clone());
                next.hypotheses.push(*q.clone());
                vec![next]
            }

            // 仮定 `∃x P(x) ⊢` を `P(x) ⊢` に変換
            Self::CasesEx { hyp } => {
                let Some(Ex { v, body, .. }) = goal.hypotheses.get(*hyp) else {
                    unreachable!()
                };
                let mut p = *body.clone();
                p.open(&Term::Var(v.clone()));
                let mut next = goal.clone();
                next.hypotheses.remove(*hyp);
                next.hypotheses.push(p);
                vec![next]
            }

            // 仮定 `⊥ ⊢` から証明完了
            Self::CasesFalse { hyp } => {
                let Some(False) = goal.hypotheses.get(*hyp) else {
                    unreachable!()
                };
                vec![]
            }

            // 仮定 `∀x P(x) ⊢` に項 `t` を代入し `P(t) ⊢` を追加
            Self::SpecializeAll { hyp, term } => {
                let Some(All { body, .. }) = goal.hypotheses.get(*hyp) else {
                    unreachable!()
                };
                let mut p = *body.clone();
                p.open(term);
                let mut next = goal.clone();
                next.hypotheses.push(p);
                vec![next]
            }

            // 仮定 `P → Q ⊢` と仮定 `P` から `Q ⊢` を追加
            Self::SpecializeTo { hyp, arg_hyp } => {
                let Some(To(_p, q)) = goal.hypotheses.get(*hyp) else {
                    unreachable!()
                };
                let To(p, _) = &goal.hypotheses[*hyp] else {
                    unreachable!()
                };
                assert!(
                    goal.hypotheses
                        .get(*arg_hyp)
                        .is_some_and(|h| h == p.as_ref()),
                    "specialize_to: argument hypothesis does not match antecedent"
                );
                let mut next = goal.clone();
                next.hypotheses.push(*q.clone());
                vec![next]
            }

            // 中間命題 `P` を導入し、その証明と利用のサブゴールを作成
            Self::Have { formula } => {
                let mut after = goal.clone();
                after.hypotheses.push(formula.clone());
                vec![
                    Goal {
                        hypotheses: goal.hypotheses.clone(),
                        target: formula.clone(),
                    },
                    after,
                ]
            }
        }
    }

    /// タクティクが適用可能かを返す。
    pub fn can_apply(&self, goal: &Goal) -> bool {
        match self {
            Self::IntroNot => matches!(goal.target, Not(_)),
            Self::IntroTo => matches!(goal.target, To(..)),
            Self::IntroAll => matches!(goal.target, All { .. }),
            Self::ConstructorAnd => matches!(goal.target, And(..)),
            Self::ConstructorIff => matches!(goal.target, Iff(..)),
            Self::Left => matches!(goal.target, Or(..)),
            Self::Right => matches!(goal.target, Or(..)),
            Self::Exists { .. } => matches!(goal.target, Ex { .. }),
            Self::Exfalso => goal.target != False,
            Self::ByContra => goal.target != False,
            Self::Assumption => goal.hypotheses.iter().any(|h| h == &goal.target),
            Self::ApplyNot { hyp } => goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, Not(_))),
            Self::ApplyTo { hyp } => goal.hypotheses.get(*hyp).is_some_and(|h| {
                matches!(h, To(_, q) if q.as_ref() == &goal.target)
            }),
            Self::ApplyIff { hyp } => goal.hypotheses.get(*hyp).is_some_and(|h| {
                matches!(h, Iff(p, q) if q.as_ref() == &goal.target || p.as_ref() == &goal.target)
            }),
            Self::CasesAnd { hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, And(..)))
            }
            Self::CasesOr { hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, Or(..)))
            }
            Self::CasesIff { hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, Iff(..)))
            }
            Self::CasesEx { hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, Ex { .. }))
            }
            Self::CasesFalse { hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, False))
            }
            Self::SpecializeAll { hyp, .. } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, All { .. }))
            }
            Self::SpecializeTo { hyp, arg_hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| match h {
                    To(p, _) => goal
                        .hypotheses
                        .get(*arg_hyp)
                        .is_some_and(|a| a == p.as_ref()),
                    _ => false,
                })
            }
            Self::Have { .. } => true,
        }
    }

    /// タクティクの表示名を返す。
    pub const fn label(&self) -> &'static str {
        match self {
            Self::IntroNot => "Intro¬",
            Self::IntroTo => "Intro→",
            Self::IntroAll => "Intro∀",
            Self::ConstructorAnd => "Conj∧",
            Self::ConstructorIff => "Conj↔",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Exists { .. } => "Exists",
            Self::Exfalso => "ExFalso",
            Self::ByContra => "ByContra",
            Self::Assumption => "Assumption",
            Self::ApplyNot { .. } => "Apply¬",
            Self::ApplyTo { .. } => "Apply→",
            Self::ApplyIff { .. } => "Apply↔",
            Self::CasesAnd { .. } => "Cases∧",
            Self::CasesOr { .. } => "Cases∨",
            Self::CasesIff { .. } => "Cases↔",
            Self::CasesEx { .. } => "Cases∃",
            Self::CasesFalse { .. } => "Cases⊥",
            Self::SpecializeAll { .. } => "Specialize∀",
            Self::SpecializeTo { .. } => "Specialize→",
            Self::Have { .. } => "Have",
        }
    }

    /// タクティクの概要を日本語で返す。
    pub const fn description(&self) -> &'static str {
        match self {
            Self::IntroNot => "ターゲットの否定を仮定に移す",
            Self::IntroTo => "含意の前件を仮定に加える",
            Self::IntroAll => "全称量化子を外して自由変数にする",
            Self::ConstructorAnd => "連言のターゲットを二つのサブゴールに分割する",
            Self::ConstructorIff => "同値のターゲットを二方向の含意に分割する",
            Self::Left => "選言の左側を選んで証明する",
            Self::Right => "選言の右側を選んで証明する",
            Self::Exists { .. } => "存在量化の証拠（witness）を与える",
            Self::Exfalso => "ターゲットを⊥に変える（爆発原理）",
            Self::ByContra => "背理法：ターゲットの否定を仮定して⊥を導く",
            Self::Assumption => "仮定のうちターゲットと一致するもので閉じる",
            Self::ApplyNot { .. } => "否定の仮定を適用し、その否定をターゲットにする",
            Self::ApplyTo { .. } => "含意の仮定を適用し、前件の証明と後件の利用に分ける",
            Self::ApplyIff { .. } => "同値の仮定をターゲットに合わせて適用する",
            Self::CasesAnd { .. } => "連言の仮定を二つの仮定に分解する",
            Self::CasesOr { .. } => "選言の仮定を場合分けする",
            Self::CasesIff { .. } => "同値の仮定を二つの含意に分解する",
            Self::CasesEx { .. } => "存在量化の仮定を具体化する",
            Self::CasesFalse { .. } => "⊥の仮定からゴールを閉じる",
            Self::SpecializeAll { .. } => "全称仮定を項で具体化する",
            Self::SpecializeTo { .. } => "含意の仮定を前件の仮定を用いて後件を導く",
            Self::Have { .. } => "中間命題を導入し、それを証明してから利用する",
        }
    }

    /// 適用前の状態を表す文字列を返す。
    /// 例えば `IntroTo` なら `"⊢ P → Q"` を返す。
    pub fn before(&self, goal: &Goal) -> String {
        match self {
            Self::IntroNot
            | Self::IntroTo
            | Self::IntroAll
            | Self::ConstructorAnd
            | Self::ConstructorIff
            | Self::Left
            | Self::Right
            | Self::Exists { .. }
            | Self::Exfalso
            | Self::ByContra
            | Self::Have { .. } => format!("⊢ {}", goal.target),

            Self::Assumption => {
                let t = goal.target.to_string();
                format!("{t} ⊢ {t}")
            }

            Self::ApplyNot { hyp }
            | Self::ApplyTo { hyp }
            | Self::ApplyIff { hyp }
            | Self::CasesAnd { hyp }
            | Self::CasesOr { hyp }
            | Self::CasesIff { hyp }
            | Self::CasesEx { hyp }
            | Self::CasesFalse { hyp }
            | Self::SpecializeAll { hyp, .. }
            | Self::SpecializeTo { hyp, .. } => {
                format!("{} ⊢", goal.hypotheses[*hyp])
            }
        }
    }

    /// 適用後の状態を表す文字列を返す。
    /// 複数のサブゴールがある場合は改行で区切る。
    pub fn after(&self, goal: &Goal) -> String {
        match self {
            Self::IntroNot => {
                let Not(p) = &goal.target else { unreachable!() };
                format!("{p} ⊢ ⊥")
            }
            Self::IntroTo => {
                let To(p, q) = &goal.target else {
                    unreachable!()
                };
                format!("{p} ⊢ {q}")
            }
            Self::IntroAll => {
                format!("⊢ {}", goal.target)
            }
            Self::ConstructorAnd => {
                let And(p, q) = &goal.target else {
                    unreachable!()
                };
                format!("⊢ {p}\n⊢ {q}")
            }
            Self::ConstructorIff => {
                let Iff(p, q) = &goal.target else {
                    unreachable!()
                };
                format!("{p} ⊢ {q}\n{q} ⊢ {p}")
            }
            Self::Left => {
                let Or(p, _) = &goal.target else {
                    unreachable!()
                };
                format!("⊢ {p}")
            }
            Self::Right => {
                let Or(_, q) = &goal.target else {
                    unreachable!()
                };
                format!("⊢ {q}")
            }
            Self::Exists { term } => {
                let Ex { body, .. } = &goal.target else {
                    unreachable!()
                };
                let mut p = *body.clone();
                p.open(term);
                format!("⊢ {p}")
            }
            Self::Exfalso => "⊢ ⊥".into(),
            Self::ByContra => {
                let p = goal.target.to_string();
                format!("¬{p} ⊢ ⊥")
            }
            Self::Assumption => String::new(),
            Self::ApplyNot { hyp } => {
                let Not(p) = &goal.hypotheses[*hyp] else {
                    unreachable!()
                };
                format!("⊢ {p}")
            }
            Self::ApplyTo { hyp } => {
                let To(p, _q) = &goal.hypotheses[*hyp] else {
                    unreachable!()
                };
                format!("⊢ {p}\n{_q} ⊢")
            }
            Self::ApplyIff { hyp } => {
                let Iff(p, q) = &goal.hypotheses[*hyp] else {
                    unreachable!()
                };
                if q.as_ref() == &goal.target {
                    format!("⊢ {p}")
                } else {
                    format!("⊢ {q}")
                }
            }
            Self::CasesAnd { hyp } => {
                let And(p, q) = &goal.hypotheses[*hyp] else {
                    unreachable!()
                };
                format!("{p}, {q} ⊢")
            }
            Self::CasesOr { hyp } => {
                let Or(p, q) = &goal.hypotheses[*hyp] else {
                    unreachable!()
                };
                format!("{p} ⊢\n{q} ⊢")
            }
            Self::CasesIff { hyp } => {
                let Iff(p, q) = &goal.hypotheses[*hyp] else {
                    unreachable!()
                };
                format!("{p}, {q} ⊢")
            }
            Self::CasesEx { hyp } => {
                let Ex { v, body, .. } = &goal.hypotheses[*hyp] else {
                    unreachable!()
                };
                let mut p = *body.clone();
                p.open(&Term::Var(v.clone()));
                format!("{p} ⊢")
            }
            Self::CasesFalse { hyp: _ } => String::new(),
            Self::SpecializeAll { hyp, term } => {
                let All { body, .. } = &goal.hypotheses[*hyp] else {
                    unreachable!()
                };
                let mut p = *body.clone();
                p.open(term);
                format!("{p} ⊢")
            }
            Self::SpecializeTo { hyp, .. } => {
                let To(_p, q) = &goal.hypotheses[*hyp] else {
                    unreachable!()
                };
                format!("{q} ⊢")
            }
            Self::Have { formula } => {
                format!("⊢ {}\n{} ⊢ {}", formula, formula, goal.target)
            }
        }
    }
}
