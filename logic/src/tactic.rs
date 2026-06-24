use crate::syntax::{Formula, Formula::*, Goal, Term};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tactic {
    // Target tactics
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
    // 仮説自動照合
    Assumption,
    // 仮説操作
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
            Tactic::IntroNot => {
                let Not(p) = &goal.target else { unreachable!() };
                let mut next = goal.clone();
                next.hypotheses.push(*p.clone());
                next.target = False;
                vec![next]
            }

            // ターゲット `⊢ P → Q` を `P ⊢ Q` に変換
            Tactic::IntroTo => {
                let To(p, q) = &goal.target else { unreachable!() };
                let mut next = goal.clone();
                next.hypotheses.push(*p.clone());
                next.target = *q.clone();
                vec![next]
            }

            // ターゲット `⊢ ∀x P(x)` を `⊢ P(x)` に変換
            Tactic::IntroAll => {
                let All { v, body, .. } = &goal.target else { unreachable!() };
                let mut p = *body.clone();
                p.open(&Term::Var(v.clone()));
                let mut next = goal.clone();
                next.target = p;
                vec![next]
            }

            // ターゲット `⊢ P ∧ Q` を `⊢ P` と `⊢ Q` に分割
            Tactic::ConstructorAnd => {
                let And(p, q) = &goal.target else { unreachable!() };
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
            Tactic::ConstructorIff => {
                let Iff(p, q) = &goal.target else { unreachable!() };
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
            Tactic::Left => {
                let Or(p, _) = &goal.target else { unreachable!() };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: *p.clone(),
                }]
            }

            // ターゲット `⊢ P ∨ Q` から右 `⊢ Q` を選択
            Tactic::Right => {
                let Or(_, q) = &goal.target else { unreachable!() };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: *q.clone(),
                }]
            }

            // ターゲット `⊢ ∃x P(x)` を `⊢ P(t)` に変換
            Tactic::Exists { term } => {
                let Ex { body, .. } = &goal.target else { unreachable!() };
                let mut p = *body.clone();
                p.open(term);
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: p,
                }]
            }

            // ターゲットを `⊥` に変更（爆発原理）
            Tactic::Exfalso => {
                assert!(goal.target != False, "exfalso: target is already ⊥");
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: False,
                }]
            }

            // ターゲット `⊢ P` を `¬P ⊢ ⊥` に変換（背理法）
            Tactic::ByContra => {
                assert!(goal.target != False, "by_contra: target is already ⊥");
                let p = goal.target.clone();
                let mut next = goal.clone();
                next.hypotheses.push(Not(Box::new(p)));
                next.target = False;
                vec![next]
            }

            // 仮説のうちゴールと一致するものがあれば証明完了
            Tactic::Assumption => {
                assert!(
                    goal.hypotheses.iter().any(|h| h == &goal.target),
                    "assumption: no matching hypothesis"
                );
                vec![]
            }

            // 仮定 `¬P ⊢` を `⊢ P` に変換
            Tactic::ApplyNot { hyp } => {
                let Some(Not(p)) = goal.hypotheses.get(*hyp) else { unreachable!() };
                vec![Goal {
                    hypotheses: goal.hypotheses.clone(),
                    target: *p.clone(),
                }]
            }

            // 仮定 `P → Q ⊢` を `⊢ P` と `Q ⊢` に変換
            Tactic::ApplyTo { hyp } => {
                let Some(To(_p, q)) = goal.hypotheses.get(*hyp) else { unreachable!() };
                assert!(q.as_ref() == &goal.target, "apply_to: conclusion does not match target");
                let To(p, _) = &goal.hypotheses[*hyp] else { unreachable!() };
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
            Tactic::ApplyIff { hyp } => {
                let Some(Iff(p, q)) = goal.hypotheses.get(*hyp) else { unreachable!() };
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
            Tactic::CasesAnd { hyp } => {
                let Some(And(p, q)) = goal.hypotheses.get(*hyp) else { unreachable!() };
                let mut next = goal.clone();
                next.hypotheses.remove(*hyp);
                next.hypotheses.push(*p.clone());
                next.hypotheses.push(*q.clone());
                vec![next]
            }

            // 仮定 `P ∨ Q ⊢` を `P ⊢` と `Q ⊢` に場合分け
            Tactic::CasesOr { hyp } => {
                let Some(Or(p, q)) = goal.hypotheses.get(*hyp) else { unreachable!() };
                let mut left = goal.clone();
                left.hypotheses.remove(*hyp);
                left.hypotheses.push(*p.clone());
                let mut right = goal.clone();
                right.hypotheses.remove(*hyp);
                right.hypotheses.push(*q.clone());
                vec![left, right]
            }

            // 仮定 `P ↔ Q ⊢` を `P, Q ⊢` に分解
            Tactic::CasesIff { hyp } => {
                let Some(Iff(p, q)) = goal.hypotheses.get(*hyp) else { unreachable!() };
                let mut next = goal.clone();
                next.hypotheses.remove(*hyp);
                next.hypotheses.push(*p.clone());
                next.hypotheses.push(*q.clone());
                vec![next]
            }

            // 仮定 `∃x P(x) ⊢` を `P(x) ⊢` に変換
            Tactic::CasesEx { hyp } => {
                let Some(Ex { v, body, .. }) = goal.hypotheses.get(*hyp) else { unreachable!() };
                let mut p = *body.clone();
                p.open(&Term::Var(v.clone()));
                let mut next = goal.clone();
                next.hypotheses.remove(*hyp);
                next.hypotheses.push(p);
                vec![next]
            }

            // 仮定 `⊥ ⊢` から証明完了
            Tactic::CasesFalse { hyp } => {
                let Some(False) = goal.hypotheses.get(*hyp) else { unreachable!() };
                vec![]
            }

            // 仮定 `∀x P(x) ⊢` に項 `t` を代入し `P(t) ⊢` を追加
            Tactic::SpecializeAll { hyp, term } => {
                let Some(All { body, .. }) = goal.hypotheses.get(*hyp) else { unreachable!() };
                let mut p = *body.clone();
                p.open(term);
                let mut next = goal.clone();
                next.hypotheses.push(p);
                vec![next]
            }

            // 仮定 `P → Q ⊢` と仮定 `P` から `Q ⊢` を追加
            Tactic::SpecializeTo { hyp, arg_hyp } => {
                let Some(To(_p, q)) = goal.hypotheses.get(*hyp) else { unreachable!() };
                let To(p, _) = &goal.hypotheses[*hyp] else { unreachable!() };
                assert!(
                    goal.hypotheses.get(*arg_hyp).is_some_and(|h| h == p.as_ref()),
                    "specialize_to: argument hypothesis does not match antecedent"
                );
                let mut next = goal.clone();
                next.hypotheses.push(*q.clone());
                vec![next]
            }

            // 中間命題 `P` を導入し、その証明と利用のサブゴールを作成
            Tactic::Have { formula } => {
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
            Tactic::IntroNot => matches!(goal.target, Not(_)),
            Tactic::IntroTo => matches!(goal.target, To(..)),
            Tactic::IntroAll => matches!(goal.target, All { .. }),
            Tactic::ConstructorAnd => matches!(goal.target, And(..)),
            Tactic::ConstructorIff => matches!(goal.target, Iff(..)),
            Tactic::Left => matches!(goal.target, Or(..)),
            Tactic::Right => matches!(goal.target, Or(..)),
            Tactic::Exists { .. } => matches!(goal.target, Ex { .. }),
            Tactic::Exfalso => goal.target != False,
            Tactic::ByContra => goal.target != False,
            Tactic::Assumption => goal.hypotheses.iter().any(|h| h == &goal.target),
            Tactic::ApplyNot { hyp } => goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, Not(_))),
            Tactic::ApplyTo { hyp } => goal.hypotheses.get(*hyp).is_some_and(|h| {
                matches!(h, To(_, q) if q.as_ref() == &goal.target)
            }),
            Tactic::ApplyIff { hyp } => goal.hypotheses.get(*hyp).is_some_and(|h| {
                matches!(h, Iff(p, q) if q.as_ref() == &goal.target || p.as_ref() == &goal.target)
            }),
            Tactic::CasesAnd { hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, And(..)))
            }
            Tactic::CasesOr { hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, Or(..)))
            }
            Tactic::CasesIff { hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, Iff(..)))
            }
            Tactic::CasesEx { hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, Ex { .. }))
            }
            Tactic::CasesFalse { hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, False))
            }
            Tactic::SpecializeAll { hyp, .. } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| matches!(h, All { .. }))
            }
            Tactic::SpecializeTo { hyp, arg_hyp } => {
                goal.hypotheses.get(*hyp).is_some_and(|h| match h {
                    To(p, _) => goal
                        .hypotheses
                        .get(*arg_hyp)
                        .is_some_and(|a| a == p.as_ref()),
                    _ => false,
                })
            }
            Tactic::Have { .. } => true,
        }
    }

    /// タクティクの表示名を返す。
    pub fn label(&self) -> &'static str {
        match self {
            Tactic::IntroNot => "Intro¬",
            Tactic::IntroTo => "Intro→",
            Tactic::IntroAll => "Intro∀",
            Tactic::ConstructorAnd => "Conj∧",
            Tactic::ConstructorIff => "Conj↔",
            Tactic::Left => "Left",
            Tactic::Right => "Right",
            Tactic::Exists { .. } => "Exists",
            Tactic::Exfalso => "ExFalso",
            Tactic::ByContra => "ByContra",
            Tactic::Assumption => "Assumption",
            Tactic::ApplyNot { .. } => "Apply¬",
            Tactic::ApplyTo { .. } => "Apply→",
            Tactic::ApplyIff { .. } => "Apply↔",
            Tactic::CasesAnd { .. } => "Cases∧",
            Tactic::CasesOr { .. } => "Cases∨",
            Tactic::CasesIff { .. } => "Cases↔",
            Tactic::CasesEx { .. } => "Cases∃",
            Tactic::CasesFalse { .. } => "Cases⊥",
            Tactic::SpecializeAll { .. } => "Specialize∀",
            Tactic::SpecializeTo { .. } => "Specialize→",
            Tactic::Have { .. } => "Have",
        }
    }

    /// タクティクの概要を日本語で返す。
    pub fn description(&self) -> &'static str {
        match self {
            Tactic::IntroNot => "ターゲットの否定を仮定に移す",
            Tactic::IntroTo => "含意の前件を仮定に加える",
            Tactic::IntroAll => "全称量化子を外して自由変数にする",
            Tactic::ConstructorAnd => "連言のターゲットを二つのサブゴールに分割する",
            Tactic::ConstructorIff => "同値のターゲットを二方向の含意に分割する",
            Tactic::Left => "選言の左側を選んで証明する",
            Tactic::Right => "選言の右側を選んで証明する",
            Tactic::Exists { .. } => "存在量化の証拠（witness）を与える",
            Tactic::Exfalso => "ターゲットを⊥に変える（爆発原理）",
            Tactic::ByContra => "背理法：ターゲットの否定を仮定して⊥を導く",
            Tactic::Assumption => "仮定のうちターゲットと一致するもので閉じる",
            Tactic::ApplyNot { .. } => "否定の仮定を適用し、その否定をターゲットにする",
            Tactic::ApplyTo { .. } => "含意の仮定を適用し、前件の証明と後件の利用に分ける",
            Tactic::ApplyIff { .. } => "同値の仮定をターゲットに合わせて適用する",
            Tactic::CasesAnd { .. } => "連言の仮定を二つの仮定に分解する",
            Tactic::CasesOr { .. } => "選言の仮定を場合分けする",
            Tactic::CasesIff { .. } => "同値の仮定を二つの含意に分解する",
            Tactic::CasesEx { .. } => "存在量化の仮定を具体化する",
            Tactic::CasesFalse { .. } => "⊥の仮定からゴールを閉じる",
            Tactic::SpecializeAll { .. } => "全称仮定を項で具体化する",
            Tactic::SpecializeTo { .. } => "含意の仮定を前件の仮定を用いて後件を導く",
            Tactic::Have { .. } => "中間命題を導入し、それを証明してから利用する",
        }
    }

    /// 適用前の状態を表す文字列を返す。
    /// 例えば `IntroTo` なら `"⊢ P → Q"` を返す。
    pub fn before(&self, goal: &Goal) -> String {
        match self {
            Tactic::IntroNot
            | Tactic::IntroTo
            | Tactic::IntroAll
            | Tactic::ConstructorAnd
            | Tactic::ConstructorIff
            | Tactic::Left
            | Tactic::Right
            | Tactic::Exists { .. }
            | Tactic::Exfalso
            | Tactic::ByContra
            | Tactic::Have { .. } => format!("⊢ {}", goal.target),

            Tactic::Assumption => {
                let t = goal.target.to_string();
                format!("{} ⊢ {}", t, t)
            }

            Tactic::ApplyNot { hyp }
            | Tactic::ApplyTo { hyp }
            | Tactic::ApplyIff { hyp }
            | Tactic::CasesAnd { hyp }
            | Tactic::CasesOr { hyp }
            | Tactic::CasesIff { hyp }
            | Tactic::CasesEx { hyp }
            | Tactic::CasesFalse { hyp }
            | Tactic::SpecializeAll { hyp, .. }
            | Tactic::SpecializeTo { hyp, .. } => {
                format!("{} ⊢", goal.hypotheses[*hyp])
            }
        }
    }

    /// 適用後の状態を表す文字列を返す。
    /// 複数のサブゴールがある場合は改行で区切る。
    pub fn after(&self, goal: &Goal) -> String {
        match self {
            Tactic::IntroNot => {
                let Not(p) = &goal.target else { unreachable!() };
                format!("{} ⊢ ⊥", p)
            }
            Tactic::IntroTo => {
                let To(p, q) = &goal.target else { unreachable!() };
                format!("{} ⊢ {}", p, q)
            }
            Tactic::IntroAll => {
                format!("⊢ {}", goal.target)
            }
            Tactic::ConstructorAnd => {
                let And(p, q) = &goal.target else { unreachable!() };
                format!("⊢ {}\n⊢ {}", p, q)
            }
            Tactic::ConstructorIff => {
                let Iff(p, q) = &goal.target else { unreachable!() };
                format!("{} ⊢ {}\n{} ⊢ {}", p, q, q, p)
            }
            Tactic::Left => {
                let Or(p, _) = &goal.target else { unreachable!() };
                format!("⊢ {}", p)
            }
            Tactic::Right => {
                let Or(_, q) = &goal.target else { unreachable!() };
                format!("⊢ {}", q)
            }
            Tactic::Exists { term } => {
                let Ex { body, .. } = &goal.target else { unreachable!() };
                let mut p = *body.clone();
                p.open(term);
                format!("⊢ {}", p)
            }
            Tactic::Exfalso => "⊢ ⊥".into(),
            Tactic::ByContra => {
                let p = goal.target.to_string();
                format!("¬{} ⊢ ⊥", p)
            }
            Tactic::Assumption => String::new(),
            Tactic::ApplyNot { hyp } => {
                let Not(p) = &goal.hypotheses[*hyp] else { unreachable!() };
                format!("⊢ {}", p)
            }
            Tactic::ApplyTo { hyp } => {
                let To(p, _q) = &goal.hypotheses[*hyp] else { unreachable!() };
                format!("⊢ {}\n{} ⊢", p, _q)
            }
            Tactic::ApplyIff { hyp } => {
                let Iff(p, q) = &goal.hypotheses[*hyp] else { unreachable!() };
                if q.as_ref() == &goal.target {
                    format!("⊢ {}", p)
                } else {
                    format!("⊢ {}", q)
                }
            }
            Tactic::CasesAnd { hyp } => {
                let And(p, q) = &goal.hypotheses[*hyp] else { unreachable!() };
                format!("{}, {} ⊢", p, q)
            }
            Tactic::CasesOr { hyp } => {
                let Or(p, q) = &goal.hypotheses[*hyp] else { unreachable!() };
                format!("{} ⊢\n{} ⊢", p, q)
            }
            Tactic::CasesIff { hyp } => {
                let Iff(p, q) = &goal.hypotheses[*hyp] else { unreachable!() };
                format!("{}, {} ⊢", p, q)
            }
            Tactic::CasesEx { hyp } => {
                let Ex { v, body, .. } = &goal.hypotheses[*hyp] else { unreachable!() };
                let mut p = *body.clone();
                p.open(&Term::Var(v.clone()));
                format!("{} ⊢", p)
            }
            Tactic::CasesFalse { hyp: _ } => String::new(),
            Tactic::SpecializeAll { hyp, term } => {
                let All { body, .. } = &goal.hypotheses[*hyp] else { unreachable!() };
                let mut p = *body.clone();
                p.open(term);
                format!("{} ⊢", p)
            }
            Tactic::SpecializeTo { hyp, .. } => {
                let To(_p, q) = &goal.hypotheses[*hyp] else { unreachable!() };
                format!("{} ⊢", q)
            }
            Tactic::Have { formula } => {
                format!("⊢ {}\n{} ⊢ {}", formula, formula, goal.target)
            }
        }
    }
}
