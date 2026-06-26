use crate::{
    parser,
    syntax::{Formula, Formula::*, Goal, Term},
    tactic::Tactic,
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct ExampleList {
    pub examples: Vec<Example>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct Example {
    pub title: String,
    pub input: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct ProofView {
    pub theorem_input: String,
    pub done: bool,
    pub title: String,
    pub subtitle: String,
    pub toolbar: ToolbarView,
    pub goal_nav: GoalNavView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub goal: Option<GoalPanelView>,
    pub selected: SelectionView,
    pub message: MessageView,
    pub available_tactics: AvailableTactics,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct AvailableTactics {
    pub target: Vec<TacticView>,
    pub hypotheses: Vec<HypothesisTactics>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct TacticView {
    pub label: String,
    pub description: String,
    pub before: String,
    pub after: String,
    pub needs_term_input: bool,
    pub needs_hypothesis_selection: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct HypothesisTactics {
    pub hypothesis_index: usize,
    pub tactics: Vec<HypothesisTacticView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct HypothesisTacticView {
    pub kind: String,
    pub label: String,
    pub description: String,
    pub before: String,
    pub after: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct ToolbarView {
    pub home_label: String,
    pub undo_label: String,
    pub redo_label: String,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct GoalNavView {
    pub current: usize,
    pub total: usize,
    pub label: String,
    pub previous_label: String,
    pub next_label: String,
    pub can_previous: bool,
    pub can_next: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct GoalPanelView {
    pub title: String,
    pub hint: String,
    pub hypotheses_title: String,
    pub no_hypotheses_text: String,
    pub target_title: String,
    pub hypotheses: Vec<HypothesisView>,
    pub target: FormulaView,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct HypothesisView {
    pub index: usize,
    pub label: String,
    pub formula: FormulaView,
    pub selected: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct FormulaView {
    pub display: String,
    pub copy: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct SelectionView {
    pub label: String,
    pub is_target: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub hypothesis_index: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct MessageView {
    pub text: String,
    pub visible: bool,
    pub is_info: bool,
    pub is_success: bool,
    pub is_error: bool,
}

#[wasm_bindgen]
pub struct Game {
    state: State,
}

#[derive(Clone, Debug)]
struct Frame {
    theorem_input: String,
    goals: Vec<Goal>,
    current: usize,
    selected: Selection,
}

#[derive(Clone, Debug)]
struct State {
    theorem_input: String,
    goals: Vec<Goal>,
    current: usize,
    selected: Selection,
    past: Vec<Frame>,
    future: Vec<Frame>,
    message: Message,
}

#[derive(Clone, Debug)]
enum Selection {
    Target,
    Hypothesis(usize),
}

#[derive(Clone, Debug)]
struct Message {
    kind: MessageKind,
    text: String,
}

#[derive(Clone, Copy, Debug)]
enum MessageKind {
    Info,
    Success,
    Error,
}

impl Message {
    fn info(text: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Info,
            text: text.into(),
        }
    }

    fn success(text: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Success,
            text: text.into(),
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Error,
            text: text.into(),
        }
    }

    fn view(&self) -> MessageView {
        MessageView {
            visible: !self.text.is_empty(),
            is_info: matches!(self.kind, MessageKind::Info),
            is_success: matches!(self.kind, MessageKind::Success),
            is_error: matches!(self.kind, MessageKind::Error),
            text: self.text.clone(),
        }
    }
}

#[wasm_bindgen]
impl Game {
    /// 入力されたシーケントからゲームを作る。
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str) -> Result<Self, JsValue> {
        Ok(Self {
            state: State::from_input(input).map_err(to_js)?,
        })
    }

    /// 現在の証明画面 `ViewModel` を返す。
    pub fn proof_view(&self) -> ProofView {
        self.state.proof_view()
    }

    pub fn select_target(&mut self) -> ProofView {
        self.state.select_target();
        self.proof_view()
    }

    pub fn select_hypothesis(&mut self, index: usize) -> ProofView {
        self.state.select_hypothesis(index);
        self.proof_view()
    }

    pub fn undo(&mut self) -> ProofView {
        self.state.undo();
        self.proof_view()
    }

    pub fn redo(&mut self) -> ProofView {
        self.state.redo();
        self.proof_view()
    }

    pub fn previous_goal(&mut self) -> ProofView {
        self.state.previous_goal();
        self.proof_view()
    }

    pub fn next_goal(&mut self) -> ProofView {
        self.state.next_goal();
        self.proof_view()
    }

    pub fn apply_intro(&mut self) -> ProofView {
        let tactic = self
            .state
            .current_goal()
            .map_or(Tactic::IntroTo, |goal| match &goal.target {
                Not(_) => Tactic::IntroNot,
                To(..) => Tactic::IntroTo,
                All { .. } => Tactic::IntroAll,
                _ => unreachable!(),
            });
        self.state.apply_tactic(tactic);
        self.proof_view()
    }

    pub fn apply_assumption(&mut self) -> ProofView {
        self.state.apply_tactic(Tactic::Assumption);
        self.proof_view()
    }

    pub fn apply_hypothesis(&mut self, hypothesis_index: usize) -> ProofView {
        let tactic = self.goal_with_hypothesis(hypothesis_index).map_or(
            Tactic::ApplyTo {
                i: hypothesis_index,
            },
            |(goal, hypothesis)| match hypothesis {
                Not(_) => Tactic::ApplyNot {
                    i: hypothesis_index,
                },
                Iff(p, q) if q.as_ref() == &goal.target || p.as_ref() == &goal.target => {
                    Tactic::ApplyIff {
                        i: hypothesis_index,
                    }
                }
                To(..) | _ => Tactic::ApplyTo {
                    i: hypothesis_index,
                },
            },
        );
        self.state.apply_tactic(tactic);
        self.proof_view()
    }

    pub fn apply_constructor(&mut self) -> ProofView {
        let tactic = self
            .state
            .current_goal()
            .map_or(Tactic::ConstructorAnd, |goal| match &goal.target {
                And(..) => Tactic::ConstructorAnd,
                Iff(..) => Tactic::ConstructorIff,
                _ => unreachable!(),
            });
        self.state.apply_tactic(tactic);
        self.proof_view()
    }

    pub fn apply_left(&mut self) -> ProofView {
        self.state.apply_tactic(Tactic::Left);
        self.proof_view()
    }

    pub fn apply_right(&mut self) -> ProofView {
        self.state.apply_tactic(Tactic::Right);
        self.proof_view()
    }

    pub fn apply_cases(&mut self, hypothesis_index: usize) -> ProofView {
        let tactic = self.goal_with_hypothesis(hypothesis_index).map_or(
            Tactic::CasesAnd {
                i: hypothesis_index,
            },
            |(_, hypothesis)| match hypothesis {
                And(..) => Tactic::CasesAnd {
                    i: hypothesis_index,
                },
                Or(..) => Tactic::CasesOr {
                    i: hypothesis_index,
                },
                Iff(..) => Tactic::CasesIff {
                    i: hypothesis_index,
                },
                Ex { .. } => Tactic::CasesEx {
                    i: hypothesis_index,
                },
                _ => unreachable!(),
            },
        );
        self.state.apply_tactic(tactic);
        self.proof_view()
    }

    pub fn apply_exists(&mut self, term: &str) -> ProofView {
        match parse_term_string(term) {
            Ok(term) => {
                self.state.apply_tactic(Tactic::Exists { t: term });
                self.proof_view()
            }
            Err(_) => self.proof_view(),
        }
    }

    pub fn apply_specialize_with_term(&mut self, hypothesis_index: usize, term: &str) -> ProofView {
        match parse_term_string(term) {
            Ok(term) => {
                self.state.apply_tactic(Tactic::SpecializeAll {
                    i: hypothesis_index,
                    t: term,
                });
                self.proof_view()
            }
            Err(_) => self.proof_view(),
        }
    }

    pub fn apply_specialize_with_hypothesis(
        &mut self,
        hypothesis_index: usize,
        _argument_index: usize,
    ) -> ProofView {
        self.state.apply_tactic(Tactic::SpecializeTo {
            i: hypothesis_index,
        });
        self.proof_view()
    }

    pub fn apply_have(&mut self, formula: &str) -> ProofView {
        match parse_formula_string(formula) {
            Ok(formula) => {
                self.state.apply_tactic(Tactic::Have { fml: formula });
                self.proof_view()
            }
            Err(_) => self.proof_view(),
        }
    }

    pub fn apply_exfalso(&mut self) -> ProofView {
        self.state.apply_tactic(Tactic::Exfalso);
        self.proof_view()
    }

    pub fn apply_by_contra(&mut self) -> ProofView {
        self.state.apply_tactic(Tactic::ByContra);
        self.proof_view()
    }

    // --- Tactic availability checks (target-side) ---

    pub fn can_intro(&self) -> bool {
        self.state.current_goal().is_some_and(|goal| {
            Tactic::IntroNot.can_apply(goal)
                || Tactic::IntroTo.can_apply(goal)
                || Tactic::IntroAll.can_apply(goal)
        })
    }

    pub fn can_constructor(&self) -> bool {
        self.state.current_goal().is_some_and(|goal| {
            Tactic::ConstructorAnd.can_apply(goal) || Tactic::ConstructorIff.can_apply(goal)
        })
    }

    pub fn can_left(&self) -> bool {
        self.state
            .current_goal()
            .is_some_and(|goal| matches!(&goal.target, Or(..)))
    }

    pub fn can_right(&self) -> bool {
        self.state
            .current_goal()
            .is_some_and(|goal| matches!(&goal.target, Or(..)))
    }

    pub fn can_exists(&self) -> bool {
        self.state
            .current_goal()
            .is_some_and(|goal| matches!(&goal.target, Ex { .. }))
    }

    pub fn can_exfalso(&self) -> bool {
        self.state
            .current_goal()
            .is_some_and(|goal| goal.target != False)
    }

    pub fn can_by_contra(&self) -> bool {
        self.state
            .current_goal()
            .is_some_and(|goal| goal.target != False)
    }

    pub fn can_assumption(&self) -> bool {
        self.state
            .current_goal()
            .is_some_and(|goal| Tactic::Assumption.can_apply(goal))
    }

    // --- Tactic availability checks (hypothesis-side) ---

    pub fn can_apply_hypothesis(&self, hypothesis_index: usize) -> bool {
        self.goal_with_hypothesis(hypothesis_index)
            .is_some_and(|(goal, hypothesis)| match hypothesis {
                Not(_) => Tactic::ApplyNot {
                    i: hypothesis_index,
                }
                .can_apply(goal),
                Iff(p, q) if q.as_ref() == &goal.target || p.as_ref() == &goal.target => {
                    Tactic::ApplyIff {
                        i: hypothesis_index,
                    }
                    .can_apply(goal)
                }
                To(..) | _ => Tactic::ApplyTo {
                    i: hypothesis_index,
                }
                .can_apply(goal),
            })
    }

    pub fn can_cases(&self, hypothesis_index: usize) -> bool {
        self.goal_with_hypothesis(hypothesis_index).is_some_and(
            |(_, hypothesis)| match hypothesis {
                And(..) | Or(..) | Iff(..) | Ex { .. } | False => true,
                _ => false,
            },
        )
    }

    pub fn can_specialize_term(&self, hypothesis_index: usize) -> bool {
        self.goal_with_hypothesis(hypothesis_index)
            .is_some_and(|(_, hypothesis)| matches!(hypothesis, All { .. }))
    }

    pub fn has_specialize_hypothesis_options(&self, hypothesis_index: usize) -> bool {
        self.state.current_goal().is_some_and(|goal| {
            let Some(To(p, _)) = goal.hypotheses.get(hypothesis_index) else {
                return false;
            };
            goal.hypotheses
                .iter()
                .enumerate()
                .any(|(i, h)| i != hypothesis_index && h == p.as_ref())
        })
    }

    pub fn is_specialize_hypothesis_enabled(
        &self,
        hypothesis_index: usize,
        argument_index: usize,
    ) -> bool {
        let Some((goal, hypothesis)) = self.goal_with_hypothesis(hypothesis_index) else {
            return false;
        };
        let To(p, _) = hypothesis else {
            return false;
        };
        goal.hypotheses
            .get(argument_index)
            .is_some_and(|a| a == p.as_ref())
    }

    // --- Descriptions ---

    pub fn intro_description(&self) -> String {
        self.state
            .current_goal()
            .map_or(String::new(), |goal| match &goal.target {
                Not(_) => Tactic::IntroNot.description().into(),
                To(..) => Tactic::IntroTo.description().into(),
                All { .. } => Tactic::IntroAll.description().into(),
                _ => String::new(),
            })
    }

    pub fn constructor_description(&self) -> String {
        self.state
            .current_goal()
            .map_or(String::new(), |goal| match &goal.target {
                And(..) => Tactic::ConstructorAnd.description().into(),
                Iff(..) => Tactic::ConstructorIff.description().into(),
                _ => String::new(),
            })
    }

    pub fn apply_hypothesis_description(&self, hypothesis_index: usize) -> String {
        self.goal_with_hypothesis(hypothesis_index)
            .map_or(String::new(), |(goal, hypothesis)| match hypothesis {
                Not(_) => Tactic::ApplyNot {
                    i: hypothesis_index,
                }
                .description()
                .into(),
                Iff(p, q) if q.as_ref() == &goal.target || p.as_ref() == &goal.target => {
                    Tactic::ApplyIff {
                        i: hypothesis_index,
                    }
                    .description()
                    .into()
                }
                To(..) | _ => Tactic::ApplyTo {
                    i: hypothesis_index,
                }
                .description()
                .into(),
            })
    }

    pub fn cases_description(&self, hypothesis_index: usize) -> String {
        self.goal_with_hypothesis(hypothesis_index)
            .map_or(String::new(), |(_, hypothesis)| match hypothesis {
                And(..) => Tactic::CasesAnd {
                    i: hypothesis_index,
                }
                .description()
                .into(),
                Or(..) => Tactic::CasesOr {
                    i: hypothesis_index,
                }
                .description()
                .into(),
                Iff(..) => Tactic::CasesIff {
                    i: hypothesis_index,
                }
                .description()
                .into(),
                Ex { .. } => Tactic::CasesEx {
                    i: hypothesis_index,
                }
                .description()
                .into(),
                _ => String::new(),
            })
    }

    pub fn exists_description(&self) -> String {
        Tactic::Exists {
            t: Term::Var("x".into()),
        }
        .description()
        .into()
    }

    pub fn exfalso_description(&self) -> String {
        Tactic::Exfalso.description().into()
    }

    pub fn by_contra_description(&self) -> String {
        Tactic::ByContra.description().into()
    }

    pub fn assumption_description(&self) -> String {
        Tactic::Assumption.description().into()
    }

    pub fn left_description(&self) -> String {
        Tactic::Left.description().into()
    }

    pub fn right_description(&self) -> String {
        Tactic::Right.description().into()
    }

    pub fn have_description(&self) -> String {
        Tactic::Have {
            fml: Atom("P".into(), vec![]),
        }
        .description()
        .into()
    }
}

impl Game {
    fn current_goal(&self) -> Option<&Goal> {
        self.state.current_goal()
    }

    fn goal_with_hypothesis(&self, hypothesis_index: usize) -> Option<(&Goal, &Formula)> {
        let goal = self.current_goal()?;
        let hypothesis = goal.hypotheses.get(hypothesis_index)?;
        Some((goal, hypothesis))
    }
}

#[wasm_bindgen]
pub fn examples() -> ExampleList {
    ExampleList {
        examples: vec![
            Example {
                title: "Implication".into(),
                input: "P, P -> Q |- Q".into(),
            },
            Example {
                title: "Conjunction elimination".into(),
                input: "P, Q, P and Q -> R and S |- R".into(),
            },
            Example {
                title: "Apply negation".into(),
                input: "P, not Q, P -> Q |- false".into(),
            },
            Example {
                title: "Universal instantiation".into(),
                input: "all x P(x) |- P(a)".into(),
            },
        ],
    }
}

impl State {
    fn from_input(input: &str) -> Result<Self, String> {
        let normalized = normalize_input(input);
        let (hyp_str, target_str) = normalized.split_once("|-").unwrap_or(("", &normalized));
        let hypotheses: Vec<Formula> = if hyp_str.trim().is_empty() {
            vec![]
        } else {
            hyp_str
                .split(',')
                .map(|s| parser::parse_formula(s.trim()).map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?
        };
        let target = if target_str.trim().is_empty() {
            False
        } else {
            parser::parse_formula(target_str.trim()).map_err(|e| e.to_string())?
        };
        Ok(Self {
            theorem_input: normalized,
            goals: vec![Goal { hypotheses, target }],
            current: 0,
            selected: Selection::Target,
            past: vec![],
            future: vec![],
            message: Message::info("Select the target or a hypothesis."),
        })
    }

    fn frame(&self) -> Frame {
        Frame {
            theorem_input: self.theorem_input.clone(),
            goals: self.goals.clone(),
            current: self.current,
            selected: self.selected.clone(),
        }
    }

    fn restore(&mut self, frame: Frame) {
        self.theorem_input = frame.theorem_input;
        self.goals = frame.goals;
        self.current = frame.current;
        self.selected = frame.selected;
        self.normalize_selection();
    }

    fn proof_view(&self) -> ProofView {
        let done = self.goals.is_empty();
        ProofView {
            theorem_input: self.theorem_input.clone(),
            done,
            title: if done { "Proof complete" } else { "Proof" }.into(),
            subtitle: if done {
                "All goals are solved."
            } else {
                "Select a formula and apply one of the available tactics."
            }
            .into(),
            toolbar: ToolbarView {
                home_label: "Home".into(),
                undo_label: "Undo".into(),
                redo_label: "Redo".into(),
                can_undo: !self.past.is_empty(),
                can_redo: !self.future.is_empty(),
            },
            goal_nav: self.goal_nav_view(),
            goal: self.current_goal().map(|goal| self.goal_panel_view(goal)),
            selected: self.selection_view(),
            message: self.message.view(),
            available_tactics: self.available_tactics(),
        }
    }

    fn available_tactics(&self) -> AvailableTactics {
        let Some(goal) = self.current_goal() else {
            return AvailableTactics {
                target: vec![],
                hypotheses: vec![],
            };
        };

        let mut target = vec![];

        // Assumption
        if Tactic::Assumption.can_apply(goal) {
            target.push(TacticView {
                label: Tactic::Assumption.label().into(),
                description: Tactic::Assumption.description().into(),
                before: Tactic::Assumption.before(goal),
                after: Tactic::Assumption.after(goal),
                needs_term_input: false,
                needs_hypothesis_selection: false,
            });
        }

        // Intro
        if Tactic::IntroNot.can_apply(goal) {
            target.push(TacticView {
                label: Tactic::IntroNot.label().into(),
                description: Tactic::IntroNot.description().into(),
                before: Tactic::IntroNot.before(goal),
                after: Tactic::IntroNot.after(goal),
                needs_term_input: false,
                needs_hypothesis_selection: false,
            });
        } else if Tactic::IntroTo.can_apply(goal) {
            target.push(TacticView {
                label: Tactic::IntroTo.label().into(),
                description: Tactic::IntroTo.description().into(),
                before: Tactic::IntroTo.before(goal),
                after: Tactic::IntroTo.after(goal),
                needs_term_input: false,
                needs_hypothesis_selection: false,
            });
        } else if Tactic::IntroAll.can_apply(goal) {
            target.push(TacticView {
                label: Tactic::IntroAll.label().into(),
                description: Tactic::IntroAll.description().into(),
                before: Tactic::IntroAll.before(goal),
                after: Tactic::IntroAll.after(goal),
                needs_term_input: false,
                needs_hypothesis_selection: false,
            });
        }

        // Constructor
        if Tactic::ConstructorAnd.can_apply(goal) {
            target.push(TacticView {
                label: Tactic::ConstructorAnd.label().into(),
                description: Tactic::ConstructorAnd.description().into(),
                before: Tactic::ConstructorAnd.before(goal),
                after: Tactic::ConstructorAnd.after(goal),
                needs_term_input: false,
                needs_hypothesis_selection: false,
            });
        } else if Tactic::ConstructorIff.can_apply(goal) {
            target.push(TacticView {
                label: Tactic::ConstructorIff.label().into(),
                description: Tactic::ConstructorIff.description().into(),
                before: Tactic::ConstructorIff.before(goal),
                after: Tactic::ConstructorIff.after(goal),
                needs_term_input: false,
                needs_hypothesis_selection: false,
            });
        }

        // Left / Right
        if matches!(&goal.target, Or(..)) {
            target.push(TacticView {
                label: Tactic::Left.label().into(),
                description: Tactic::Left.description().into(),
                before: Tactic::Left.before(goal),
                after: Tactic::Left.after(goal),
                needs_term_input: false,
                needs_hypothesis_selection: false,
            });
            target.push(TacticView {
                label: Tactic::Right.label().into(),
                description: Tactic::Right.description().into(),
                before: Tactic::Right.before(goal),
                after: Tactic::Right.after(goal),
                needs_term_input: false,
                needs_hypothesis_selection: false,
            });
        }

        // Exists
        if matches!(&goal.target, Ex { .. }) {
            target.push(TacticView {
                label: Tactic::Exists {
                    t: Term::Var("x".into()),
                }
                .label()
                .into(),
                description: Tactic::Exists {
                    t: Term::Var("x".into()),
                }
                .description()
                .into(),
                before: Tactic::Exists {
                    t: Term::Var("x".into()),
                }
                .before(goal),
                after: Tactic::Exists {
                    t: Term::Var("x".into()),
                }
                .after(goal),
                needs_term_input: true,
                needs_hypothesis_selection: false,
            });
        }

        // Exfalso
        if goal.target != False {
            target.push(TacticView {
                label: Tactic::Exfalso.label().into(),
                description: Tactic::Exfalso.description().into(),
                before: Tactic::Exfalso.before(goal),
                after: Tactic::Exfalso.after(goal),
                needs_term_input: false,
                needs_hypothesis_selection: false,
            });
        }

        // ByContra
        if goal.target != False {
            target.push(TacticView {
                label: Tactic::ByContra.label().into(),
                description: Tactic::ByContra.description().into(),
                before: Tactic::ByContra.before(goal),
                after: Tactic::ByContra.after(goal),
                needs_term_input: false,
                needs_hypothesis_selection: false,
            });
        }

        // Hypothesis tactics per hypothesis
        let hypotheses: Vec<HypothesisTactics> = if matches!(self.selected, Selection::Target) {
            vec![]
        } else {
            goal.hypotheses
                .iter()
                .enumerate()
                .filter_map(|(i, hyp)| {
                    let mut tactics = vec![];

                    // Apply
                    let apply_tactic: Option<Tactic> = match hyp {
                        Not(_) => Some(Tactic::ApplyNot { i }),
                        Iff(p, q) if q.as_ref() == &goal.target || p.as_ref() == &goal.target => {
                            Some(Tactic::ApplyIff { i })
                        }
                        To(..) => Some(Tactic::ApplyTo { i }),
                        _ => None,
                    };
                    if let Some(tactic) = apply_tactic
                        && tactic.can_apply(goal)
                    {
                        tactics.push(HypothesisTacticView {
                            kind: "apply".into(),
                            label: tactic.label().into(),
                            description: tactic.description().into(),
                            before: tactic.before(goal),
                            after: tactic.after(goal),
                        });
                    }

                    // Cases
                    let cases_tactic: Option<Tactic> = match hyp {
                        And(..) => Some(Tactic::CasesAnd { i }),
                        Or(..) => Some(Tactic::CasesOr { i }),
                        Iff(..) => Some(Tactic::CasesIff { i }),
                        Ex { .. } => Some(Tactic::CasesEx { i }),
                        _ => None,
                    };
                    if let Some(tactic) = cases_tactic {
                        tactics.push(HypothesisTacticView {
                            kind: "cases".into(),
                            label: tactic.label().into(),
                            description: tactic.description().into(),
                            before: tactic.before(goal),
                            after: tactic.after(goal),
                        });
                    }

                    // SpecializeAll (∀)
                    if matches!(hyp, All { .. }) {
                        let tactic = Tactic::SpecializeAll {
                            i,
                            t: Term::Var("x".into()),
                        };
                        tactics.push(HypothesisTacticView {
                            kind: "specialize_term".into(),
                            label: tactic.label().into(),
                            description: tactic.description().into(),
                            before: tactic.before(goal),
                            after: tactic.after(goal),
                        });
                    }

                    // SpecializeTo (→ with matching hypothesis)
                    if let To(p, _) = hyp {
                        for (arg_i, arg_hyp) in goal.hypotheses.iter().enumerate() {
                            if arg_i != i && arg_hyp == p.as_ref() {
                                let tactic = Tactic::SpecializeTo { i };
                                tactics.push(HypothesisTacticView {
                                    kind: "specialize_hypothesis".into(),
                                    label: tactic.label().into(),
                                    description: tactic.description().into(),
                                    before: tactic.before(goal),
                                    after: tactic.after(goal),
                                });
                            }
                        }
                    }

                    if tactics.is_empty() {
                        None
                    } else {
                        Some(HypothesisTactics {
                            hypothesis_index: i,
                            tactics,
                        })
                    }
                })
                .collect()
        };

        AvailableTactics { target, hypotheses }
    }

    fn current_goal(&self) -> Option<&Goal> {
        self.goals.get(self.current)
    }

    fn goal_nav_view(&self) -> GoalNavView {
        let total = self.goals.len();
        GoalNavView {
            current: if total == 0 { 0 } else { self.current + 1 },
            total,
            label: if total == 0 {
                "No remaining goals".into()
            } else {
                format!("Goal {} of {total}", self.current + 1)
            },
            previous_label: "Previous goal".into(),
            next_label: "Next goal".into(),
            can_previous: self.current > 0,
            can_next: self.current + 1 < total,
        }
    }

    fn goal_panel_view(&self, goal: &Goal) -> GoalPanelView {
        GoalPanelView {
            title: "Current goal".into(),
            hint: "Select the target or one hypothesis. Only applicable tactics are shown.".into(),
            hypotheses_title: "Hypotheses".into(),
            no_hypotheses_text: "No hypotheses.".into(),
            target_title: "Target".into(),
            hypotheses: goal
                .hypotheses
                .iter()
                .enumerate()
                .map(|(index, formula)| HypothesisView {
                    index,
                    label: "Hypothesis".into(),
                    formula: formula_view(formula),
                    selected: matches!(self.selected, Selection::Hypothesis(i) if i == index),
                })
                .collect(),
            target: formula_view(&goal.target),
        }
    }

    fn selection_view(&self) -> SelectionView {
        match self.selected {
            Selection::Target => SelectionView {
                label: "Target tactics".into(),
                is_target: true,
                hypothesis_index: None,
            },
            Selection::Hypothesis(index) => SelectionView {
                label: format!("Hypothesis {} tactics", index + 1),
                is_target: false,
                hypothesis_index: Some(index),
            },
        }
    }

    fn apply_tactic(&mut self, tactic: Tactic) {
        if self.goals.is_empty() {
            self.message = Message::info("The proof is already complete.");
            return;
        }

        let before = self.frame();
        let goal = self.goals.remove(self.current);
        let new_goals = tactic.apply(&goal);
        for (i, goal) in new_goals.into_iter().enumerate() {
            self.goals.insert(self.current + i, goal);
        }
        self.past.push(before);
        self.future.clear();
        self.normalize_current();
        self.selected = Selection::Target;
        self.message = if self.goals.is_empty() {
            Message::success("Proof complete.")
        } else {
            Message::success(format!("Applied {}.", tactic.label()))
        };
    }

    fn undo(&mut self) {
        if let Some(frame) = self.past.pop() {
            let current = self.frame();
            self.future.push(current);
            self.restore(frame);
            self.message = Message::info("Undone.");
        } else {
            self.message = Message::info("Nothing to undo.");
        }
    }

    fn redo(&mut self) {
        if let Some(frame) = self.future.pop() {
            let current = self.frame();
            self.past.push(current);
            self.restore(frame);
            self.message = Message::info("Redone.");
        } else {
            self.message = Message::info("Nothing to redo.");
        }
    }

    fn select_target(&mut self) {
        self.selected = Selection::Target;
        self.message = Message::info("Target selected.");
    }

    fn select_hypothesis(&mut self, index: usize) {
        if self
            .current_goal()
            .is_some_and(|goal| index < goal.hypotheses.len())
        {
            self.selected = Selection::Hypothesis(index);
            self.message = Message::info(format!("Hypothesis {} selected.", index + 1));
        } else {
            self.message = Message::error("Selected hypothesis does not exist.");
        }
    }

    fn previous_goal(&mut self) {
        if self.current > 0 {
            self.current -= 1;
            self.selected = Selection::Target;
            self.message = Message::info(format!("Goal {} selected.", self.current + 1));
        } else {
            self.message = Message::info("Already at the first goal.");
        }
    }

    fn next_goal(&mut self) {
        if self.current + 1 < self.goals.len() {
            self.current += 1;
            self.selected = Selection::Target;
            self.message = Message::info(format!("Goal {} selected.", self.current + 1));
        } else {
            self.message = Message::info("Already at the last goal.");
        }
    }

    fn normalize_current(&mut self) {
        if self.goals.is_empty() {
            self.current = 0;
        } else if self.current >= self.goals.len() {
            self.current = self.goals.len() - 1;
        }
        self.normalize_selection();
    }

    fn normalize_selection(&mut self) {
        if let Selection::Hypothesis(index) = self.selected
            && self
                .current_goal()
                .is_none_or(|goal| index >= goal.hypotheses.len())
        {
            self.selected = Selection::Target;
        }
    }
}

fn formula_view(formula: &Formula) -> FormulaView {
    let display = formula.to_string();
    FormulaView {
        copy: display.clone(),
        display,
    }
}

fn parse_term_string(source: &str) -> Result<Term, String> {
    parser::parse_term(&normalize_input(source)).map_err(|e| e.to_string())
}

fn parse_formula_string(source: &str) -> Result<Formula, String> {
    parser::parse_formula(&normalize_input(source)).map_err(|e| e.to_string())
}

fn normalize_input(input: &str) -> String {
    input
        .trim()
        .replace('∧', "and")
        .replace('∨', "or")
        .replace('¬', "not ")
        .replace(['→', '⇒'], "->")
        .replace('⊢', "|-")
        .replace('⊥', "false")
}

fn to_js(e: impl ToString) -> JsValue {
    JsValue::from_str(&e.to_string())
}
