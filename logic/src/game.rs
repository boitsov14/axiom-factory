use crate::{
    parser,
    syntax::{Formula, Formula::*, Goal, Term},
    tactic::{Arg, Tactic},
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

const DEFAULT_INPUT: &str = "P, Q, P and Q -> R and S |- R";

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
pub struct AppView {
    pub home: HomeView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub proof: Option<ProofView>,
    pub message: MessageView,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct HomeView {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub input: String,
    pub input_label: String,
    pub syntax_hint: String,
    pub start_label: String,
    pub examples_title: String,
    pub examples_hint: String,
    pub examples: Vec<Example>,
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
    pub action_title: String,
    pub action_hint: String,
    pub no_target_tactics_text: String,
    pub no_hypothesis_tactics_text: String,
    pub have_title: String,
    pub have_hint: String,
    pub have_placeholder: String,
    pub have_add_label: String,
    pub have_unavailable_text: String,
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

#[wasm_bindgen]
pub struct App {
    state: AppState,
}

#[derive(Clone, Debug)]
struct AppState {
    screen: Screen,
    home_input: String,
    message: Message,
}

#[derive(Clone, Debug)]
enum Screen {
    Home,
    Proof(State),
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            state: AppState {
                screen: Screen::Home,
                home_input: DEFAULT_INPUT.into(),
                message: Message::info("Enter a theorem or choose an example."),
            },
        }
    }

    pub fn view(&self) -> AppView {
        self.state.view()
    }

    pub fn set_input(&mut self, input: &str) -> AppView {
        self.state.set_input(input);
        self.view()
    }

    pub fn start_proof(&mut self, input: &str) -> AppView {
        self.state.start_proof(input);
        self.view()
    }

    pub fn choose_example(&mut self, index: usize) -> AppView {
        self.state.choose_example(index);
        self.view()
    }

    pub fn open_home(&mut self) -> AppView {
        self.state.open_home();
        self.view()
    }

    pub fn select_target(&mut self) -> AppView {
        self.state.apply_to_proof(State::select_target);
        self.view()
    }

    pub fn select_hypothesis(&mut self, index: usize) -> AppView {
        self.state
            .apply_to_proof(|proof| proof.select_hypothesis(index));
        self.view()
    }

    pub fn undo(&mut self) -> AppView {
        self.state.apply_to_proof(State::undo);
        self.view()
    }

    pub fn redo(&mut self) -> AppView {
        self.state.apply_to_proof(State::redo);
        self.view()
    }

    pub fn previous_goal(&mut self) -> AppView {
        self.state.apply_to_proof(State::previous_goal);
        self.view()
    }

    pub fn next_goal(&mut self) -> AppView {
        self.state.apply_to_proof(State::next_goal);
        self.view()
    }

    pub fn apply_intro(&mut self) -> AppView {
        self.state.apply_tactic(Tactic::Intro);
        self.view()
    }

    pub fn apply_assumption(&mut self, hypothesis_index: usize) -> AppView {
        self.state.apply_tactic(Tactic::Assumption {
            hyp: hypothesis_index,
        });
        self.view()
    }

    pub fn apply_hypothesis(&mut self, hypothesis_index: usize) -> AppView {
        self.state.apply_tactic(Tactic::Apply {
            hyp: hypothesis_index,
        });
        self.view()
    }

    pub fn apply_constructor(&mut self) -> AppView {
        self.state.apply_tactic(Tactic::Constructor);
        self.view()
    }

    pub fn apply_left(&mut self) -> AppView {
        self.state.apply_tactic(Tactic::Left);
        self.view()
    }

    pub fn apply_right(&mut self) -> AppView {
        self.state.apply_tactic(Tactic::Right);
        self.view()
    }

    pub fn apply_cases(&mut self, hypothesis_index: usize) -> AppView {
        self.state.apply_tactic(Tactic::Cases {
            hyp: hypothesis_index,
        });
        self.view()
    }

    pub fn apply_exists(&mut self, term: &str) -> AppView {
        self.state.apply_exists(term);
        self.view()
    }

    pub fn apply_specialize_with_term(&mut self, hypothesis_index: usize, term: &str) -> AppView {
        self.state
            .apply_specialize_with_term(hypothesis_index, term);
        self.view()
    }

    pub fn apply_specialize_with_hypothesis(
        &mut self,
        hypothesis_index: usize,
        argument_index: usize,
    ) -> AppView {
        self.state.apply_tactic(Tactic::Specialize {
            hyp: hypothesis_index,
            arg: Arg::Hyp(argument_index),
        });
        self.view()
    }

    pub fn apply_have(&mut self, formula: &str) -> AppView {
        self.state.apply_have(formula);
        self.view()
    }

    pub fn apply_exfalso(&mut self) -> AppView {
        self.state.apply_tactic(Tactic::Exfalso);
        self.view()
    }

    pub fn apply_by_contra(&mut self) -> AppView {
        self.state.apply_tactic(Tactic::ByContra);
        self.view()
    }
}

impl App {
    fn current_goal(&self) -> Option<&Goal> {
        match &self.state.screen {
            Screen::Home => None,
            Screen::Proof(proof) => proof.current_goal(),
        }
    }

    fn current_hypothesis(&self, hypothesis_index: usize) -> Option<(&Goal, &Formula)> {
        let goal = self.current_goal()?;
        let hypothesis = goal.hypotheses.get(hypothesis_index)?;
        Some((goal, hypothesis))
    }

    fn apply_tactic_and_view(&mut self, tactic: Tactic) -> AppView {
        self.state.apply_tactic(tactic);
        self.view()
    }

    fn apply_exists_and_view(&mut self, term: &str) -> AppView {
        self.state.apply_exists(term);
        self.view()
    }

    fn apply_specialize_with_term_and_view(
        &mut self,
        hypothesis_index: usize,
        term: &str,
    ) -> AppView {
        self.state
            .apply_specialize_with_term(hypothesis_index, term);
        self.view()
    }

    fn apply_have_and_view(&mut self, formula: &str) -> AppView {
        self.state.apply_have(formula);
        self.view()
    }
}

#[wasm_bindgen]
pub struct IntroTactic;

#[wasm_bindgen]
impl IntroTactic {
    pub fn label() -> String {
        "Intro".into()
    }

    pub fn description(_app: &App) -> String {
        "Introduce an implication, universal quantifier, or negation.".into()
    }

    pub fn is_enabled(app: &App) -> bool {
        app.current_goal()
            .is_some_and(|goal| matches!(&goal.target, To(..) | All { .. } | Not(_)))
    }

    pub fn apply(app: &mut App) -> AppView {
        app.apply_tactic_and_view(Tactic::Intro)
    }
}

#[wasm_bindgen]
pub struct AssumptionTactic;

#[wasm_bindgen]
impl AssumptionTactic {
    pub fn label() -> String {
        "Assumption".into()
    }

    pub fn description(_app: &App, _hypothesis_index: usize) -> String {
        "Close the goal with this matching hypothesis.".into()
    }

    pub fn is_enabled(app: &App, hypothesis_index: usize) -> bool {
        app.current_hypothesis(hypothesis_index)
            .is_some_and(|(goal, hypothesis)| hypothesis == &goal.target)
    }

    pub fn apply(app: &mut App, hypothesis_index: usize) -> AppView {
        app.apply_tactic_and_view(Tactic::Assumption {
            hyp: hypothesis_index,
        })
    }
}

#[wasm_bindgen]
pub struct ApplyTactic;

#[wasm_bindgen]
impl ApplyTactic {
    pub fn label() -> String {
        "Apply".into()
    }

    pub fn description(_app: &App, _hypothesis_index: usize) -> String {
        "Apply this implication, negation, or equivalence hypothesis to the target.".into()
    }

    pub fn is_enabled(app: &App, hypothesis_index: usize) -> bool {
        app.current_hypothesis(hypothesis_index)
            .is_some_and(|(goal, hypothesis)| can_apply(hypothesis, &goal.target))
    }

    pub fn apply(app: &mut App, hypothesis_index: usize) -> AppView {
        app.apply_tactic_and_view(Tactic::Apply {
            hyp: hypothesis_index,
        })
    }
}

#[wasm_bindgen]
pub struct ConstructorTactic;

#[wasm_bindgen]
impl ConstructorTactic {
    pub fn label() -> String {
        "Constructor".into()
    }

    pub fn description(_app: &App) -> String {
        "Split a conjunction or equivalence target into subgoals.".into()
    }

    pub fn is_enabled(app: &App) -> bool {
        app.current_goal()
            .is_some_and(|goal| matches!(&goal.target, And(..) | Iff(..)))
    }

    pub fn apply(app: &mut App) -> AppView {
        app.apply_tactic_and_view(Tactic::Constructor)
    }
}

#[wasm_bindgen]
pub struct LeftTactic;

#[wasm_bindgen]
impl LeftTactic {
    pub fn label() -> String {
        "Left".into()
    }

    pub fn description(_app: &App) -> String {
        "Prove the left side of a disjunction target.".into()
    }

    pub fn is_enabled(app: &App) -> bool {
        app.current_goal()
            .is_some_and(|goal| matches!(&goal.target, Or(..)))
    }

    pub fn apply(app: &mut App) -> AppView {
        app.apply_tactic_and_view(Tactic::Left)
    }
}

#[wasm_bindgen]
pub struct RightTactic;

#[wasm_bindgen]
impl RightTactic {
    pub fn label() -> String {
        "Right".into()
    }

    pub fn description(_app: &App) -> String {
        "Prove the right side of a disjunction target.".into()
    }

    pub fn is_enabled(app: &App) -> bool {
        app.current_goal()
            .is_some_and(|goal| matches!(&goal.target, Or(..)))
    }

    pub fn apply(app: &mut App) -> AppView {
        app.apply_tactic_and_view(Tactic::Right)
    }
}

#[wasm_bindgen]
pub struct CasesTactic;

#[wasm_bindgen]
impl CasesTactic {
    pub fn label() -> String {
        "Cases".into()
    }

    pub fn description(_app: &App, _hypothesis_index: usize) -> String {
        "Split this hypothesis or eliminate falsehood.".into()
    }

    pub fn is_enabled(app: &App, hypothesis_index: usize) -> bool {
        app.current_hypothesis(hypothesis_index)
            .is_some_and(|(_, hypothesis)| {
                matches!(hypothesis, And(..) | Or(..) | Iff(..) | Ex { .. } | False)
            })
    }

    pub fn apply(app: &mut App, hypothesis_index: usize) -> AppView {
        app.apply_tactic_and_view(Tactic::Cases {
            hyp: hypothesis_index,
        })
    }
}

#[wasm_bindgen]
pub struct ExistsTactic;

#[wasm_bindgen]
impl ExistsTactic {
    pub fn label() -> String {
        "Exists".into()
    }

    pub fn description(_app: &App) -> String {
        "Provide a witness for the existential target.".into()
    }

    pub fn is_available(app: &App) -> bool {
        app.current_goal()
            .is_some_and(|goal| matches!(&goal.target, Ex { .. }))
    }

    pub fn can_apply(app: &App, term: &str) -> bool {
        Self::is_available(app) && parse_term_string(term).is_ok()
    }

    pub fn apply(app: &mut App, term: &str) -> AppView {
        app.apply_exists_and_view(term)
    }
}

#[wasm_bindgen]
pub struct SpecializeTermTactic;

#[wasm_bindgen]
impl SpecializeTermTactic {
    pub fn label() -> String {
        "Specialize".into()
    }

    pub fn description(_app: &App, _hypothesis_index: usize) -> String {
        "Instantiate this universal hypothesis with a term.".into()
    }

    pub fn is_available(app: &App, hypothesis_index: usize) -> bool {
        app.current_hypothesis(hypothesis_index)
            .is_some_and(|(_, hypothesis)| matches!(hypothesis, All { .. }))
    }

    pub fn can_apply(app: &App, hypothesis_index: usize, term: &str) -> bool {
        Self::is_available(app, hypothesis_index) && parse_term_string(term).is_ok()
    }

    pub fn apply(app: &mut App, hypothesis_index: usize, term: &str) -> AppView {
        app.apply_specialize_with_term_and_view(hypothesis_index, term)
    }
}

#[wasm_bindgen]
pub struct SpecializeHypothesisTactic;

#[wasm_bindgen]
impl SpecializeHypothesisTactic {
    pub fn label() -> String {
        "Specialize".into()
    }

    pub fn description(_app: &App, _hypothesis_index: usize, argument_index: usize) -> String {
        format!(
            "Instantiate this implication using hypothesis {}.",
            argument_index + 1
        )
    }

    pub fn is_enabled(app: &App, hypothesis_index: usize, argument_index: usize) -> bool {
        let Some((goal, hypothesis)) = app.current_hypothesis(hypothesis_index) else {
            return false;
        };
        let To(p, _) = hypothesis else {
            return false;
        };
        goal.hypotheses
            .get(argument_index)
            .is_some_and(|argument| argument == p.as_ref())
    }

    pub fn has_options(app: &App, hypothesis_index: usize) -> bool {
        app.current_goal().is_some_and(|goal| {
            goal.hypotheses
                .iter()
                .enumerate()
                .any(|(argument_index, _)| Self::is_enabled(app, hypothesis_index, argument_index))
        })
    }

    pub fn apply(app: &mut App, hypothesis_index: usize, argument_index: usize) -> AppView {
        app.apply_tactic_and_view(Tactic::Specialize {
            hyp: hypothesis_index,
            arg: Arg::Hyp(argument_index),
        })
    }
}

#[wasm_bindgen]
pub struct HaveTactic;

#[wasm_bindgen]
impl HaveTactic {
    pub fn label() -> String {
        "Have".into()
    }

    pub fn description(_app: &App) -> String {
        "Create a separate subgoal and add it as a later hypothesis.".into()
    }

    pub fn is_available(app: &App) -> bool {
        app.current_goal().is_some()
    }

    pub fn can_apply(app: &App, formula: &str) -> bool {
        Self::is_available(app) && parse_formula_string(formula).is_ok()
    }

    pub fn apply(app: &mut App, formula: &str) -> AppView {
        app.apply_have_and_view(formula)
    }
}

#[wasm_bindgen]
pub struct ExfalsoTactic;

#[wasm_bindgen]
impl ExfalsoTactic {
    pub fn label() -> String {
        "Exfalso".into()
    }

    pub fn description(_app: &App) -> String {
        "Reduce the current target to False.".into()
    }

    pub fn is_enabled(app: &App) -> bool {
        app.current_goal()
            .is_some_and(|goal| !matches!(&goal.target, False))
    }

    pub fn apply(app: &mut App) -> AppView {
        app.apply_tactic_and_view(Tactic::Exfalso)
    }
}

#[wasm_bindgen]
pub struct ByContraTactic;

#[wasm_bindgen]
impl ByContraTactic {
    pub fn label() -> String {
        "By Contra".into()
    }

    pub fn description(_app: &App) -> String {
        "Prove the target by contradiction.".into()
    }

    pub fn is_enabled(app: &App) -> bool {
        app.current_goal()
            .is_some_and(|goal| !matches!(&goal.target, False))
    }

    pub fn apply(app: &mut App) -> AppView {
        app.apply_tactic_and_view(Tactic::ByContra)
    }
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
    ///
    /// # Errors
    ///
    /// 入力がシーケントとして構文解析できない場合にエラーを返す。
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str) -> Result<Self, JsValue> {
        Ok(Self {
            state: State::from_input(input).map_err(to_js)?,
        })
    }

    /// 現在の証明画面 ViewModel を返す。
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
        self.state.apply_tactic(Tactic::Intro);
        self.proof_view()
    }

    pub fn apply_assumption(&mut self, hypothesis_index: usize) -> ProofView {
        self.state.apply_tactic(Tactic::Assumption {
            hyp: hypothesis_index,
        });
        self.proof_view()
    }

    pub fn apply_hypothesis(&mut self, hypothesis_index: usize) -> ProofView {
        self.state.apply_tactic(Tactic::Apply {
            hyp: hypothesis_index,
        });
        self.proof_view()
    }

    pub fn apply_constructor(&mut self) -> ProofView {
        self.state.apply_tactic(Tactic::Constructor);
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
        self.state.apply_tactic(Tactic::Cases {
            hyp: hypothesis_index,
        });
        self.proof_view()
    }

    pub fn apply_exists(&mut self, term: &str) -> ProofView {
        self.state.apply_exists(term);
        self.proof_view()
    }

    pub fn apply_specialize_with_term(&mut self, hypothesis_index: usize, term: &str) -> ProofView {
        self.state
            .apply_specialize_with_term(hypothesis_index, term);
        self.proof_view()
    }

    pub fn apply_specialize_with_hypothesis(
        &mut self,
        hypothesis_index: usize,
        argument_index: usize,
    ) -> ProofView {
        self.state.apply_tactic(Tactic::Specialize {
            hyp: hypothesis_index,
            arg: Arg::Hyp(argument_index),
        });
        self.proof_view()
    }

    pub fn apply_have(&mut self, formula: &str) -> ProofView {
        self.state.apply_have(formula);
        self.proof_view()
    }

    pub fn apply_exfalso(&mut self) -> ProofView {
        self.state.apply_tactic(Tactic::Exfalso);
        self.proof_view()
    }

    pub fn apply_by_contra(&mut self) -> ProofView {
        self.state.apply_tactic(Tactic::ByContra);
        self.proof_view()
    }
}

/// サンプル問題を返す。
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

impl AppState {
    fn set_input(&mut self, input: &str) {
        self.home_input = input.to_owned();
    }

    fn start_proof(&mut self, input: &str) {
        let normalized = normalize_input(input);
        self.home_input = normalized.clone();
        match State::from_input(&normalized) {
            Ok(state) => {
                self.screen = Screen::Proof(state);
                self.message = Message::success("Proof started.");
            }
            Err(e) => {
                self.screen = Screen::Home;
                self.message = Message::error(e);
            }
        }
    }

    fn choose_example(&mut self, index: usize) {
        let list = examples();
        if let Some(example) = list.examples.get(index) {
            self.start_proof(&example.input);
        } else {
            self.message = Message::error("Selected example does not exist.");
        }
    }

    fn open_home(&mut self) {
        self.screen = Screen::Home;
        self.message = Message::info("Returned to the home screen.");
    }

    fn apply_to_proof(&mut self, op: impl FnOnce(&mut State)) {
        match &mut self.screen {
            Screen::Home => {
                self.message = Message::error("Start a proof before applying proof actions.");
            }
            Screen::Proof(proof) => {
                op(proof);
                self.message = proof.message.clone();
            }
        }
    }

    fn apply_tactic(&mut self, tactic: Tactic) {
        self.apply_to_proof(|proof| {
            if let Err(e) = proof.apply_tactic(tactic) {
                proof.message = Message::error(e);
            }
        });
    }

    fn apply_exists(&mut self, term: &str) {
        match parse_term_string(term) {
            Ok(term) => self.apply_tactic(Tactic::Exists { term }),
            Err(e) => self.message = Message::error(e),
        }
    }

    fn apply_specialize_with_term(&mut self, hypothesis_index: usize, term: &str) {
        match parse_term_string(term) {
            Ok(term) => self.apply_tactic(Tactic::Specialize {
                hyp: hypothesis_index,
                arg: Arg::Term(term),
            }),
            Err(e) => self.message = Message::error(e),
        }
    }

    fn apply_have(&mut self, formula: &str) {
        match parse_formula_string(formula) {
            Ok(formula) => self.apply_tactic(Tactic::Have { formula }),
            Err(e) => self.message = Message::error(e),
        }
    }

    fn view(&self) -> AppView {
        AppView {
            home: self.home_view(),
            proof: match &self.screen {
                Screen::Home => None,
                Screen::Proof(proof) => Some(proof.proof_view()),
            },
            message: self.message.view(),
        }
    }

    fn home_view(&self) -> HomeView {
        HomeView {
            title: "Axiom Factory".into(),
            subtitle: "Build proofs by applying tactics.".into(),
            description: "Enter a propositional sequent or choose an example. Rust owns the game state and returns a renderable view model.".into(),
            input: self.home_input.clone(),
            input_label: "Theorem".into(),
            syntax_hint: "Use ASCII syntax such as P, P -> Q |- Q.".into(),
            start_label: "Start proof".into(),
            examples_title: "Examples".into(),
            examples_hint: "Choose one to open the proof screen.".into(),
            examples: examples().examples,
        }
    }
}

impl State {
    /// 入力文字列から状態を作る。
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

    /// 現在の状態を保存用フレームに変換する。
    fn frame(&self) -> Frame {
        Frame {
            theorem_input: self.theorem_input.clone(),
            goals: self.goals.clone(),
            current: self.current,
            selected: self.selected.clone(),
        }
    }

    /// 保存用フレームを現在状態に復元する。
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
            action_title: self.selection_view().label,
            action_hint:
                "Tactics are Rust methods. This panel only calls applicable static tactic APIs."
                    .into(),
            no_target_tactics_text: "No applicable target tactics.".into(),
            no_hypothesis_tactics_text: "No applicable tactics for the selected hypothesis.".into(),
            have_title: "Have".into(),
            have_hint: "Introduce a separate intermediate claim.".into(),
            have_placeholder: "formula".into(),
            have_add_label: "Add".into(),
            have_unavailable_text: "Have is not available.".into(),
        }
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

    fn apply_exists(&mut self, term: &str) {
        match parse_term_string(term) {
            Ok(term) => {
                if let Err(e) = self.apply_tactic(Tactic::Exists { term }) {
                    self.message = Message::error(e);
                }
            }
            Err(e) => self.message = Message::error(e),
        }
    }

    fn apply_specialize_with_term(&mut self, hypothesis_index: usize, term: &str) {
        match parse_term_string(term) {
            Ok(term) => {
                if let Err(e) = self.apply_tactic(Tactic::Specialize {
                    hyp: hypothesis_index,
                    arg: Arg::Term(term),
                }) {
                    self.message = Message::error(e);
                }
            }
            Err(e) => self.message = Message::error(e),
        }
    }

    fn apply_have(&mut self, formula: &str) {
        match parse_formula_string(formula) {
            Ok(formula) => {
                if let Err(e) = self.apply_tactic(Tactic::Have { formula }) {
                    self.message = Message::error(e);
                }
            }
            Err(e) => self.message = Message::error(e),
        }
    }

    /// タクティクを適用する。
    fn apply_tactic(&mut self, tactic: Tactic) -> Result<(), String> {
        if self.goals.is_empty() {
            self.message = Message::info("The proof is already complete.");
            return Ok(());
        }

        let before = self.frame();
        let goal = self.goals.remove(self.current);
        let new_goals = tactic.apply(goal)?;
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
        Ok(())
    }

    /// 直前の証明状態に戻す。
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

    /// `undo` した証明状態を再適用する。
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

    /// 一つ前のゴールを現在ゴールにする。
    fn previous_goal(&mut self) {
        if self.current > 0 {
            self.current -= 1;
            self.selected = Selection::Target;
            self.message = Message::info(format!("Goal {} selected.", self.current + 1));
        } else {
            self.message = Message::info("Already at the first goal.");
        }
    }

    /// 一つ後のゴールを現在ゴールにする。
    fn next_goal(&mut self) {
        if self.current + 1 < self.goals.len() {
            self.current += 1;
            self.selected = Selection::Target;
            self.message = Message::info(format!("Goal {} selected.", self.current + 1));
        } else {
            self.message = Message::info("Already at the last goal.");
        }
    }

    /// 現在ゴール番号を有効範囲に収める。
    fn normalize_current(&mut self) {
        if self.goals.is_empty() {
            self.current = 0;
        } else if self.current >= self.goals.len() {
            self.current = self.goals.len() - 1;
        }
        self.normalize_selection();
    }

    fn normalize_selection(&mut self) {
        if let Selection::Hypothesis(index) = self.selected {
            if !self
                .current_goal()
                .is_some_and(|goal| index < goal.hypotheses.len())
            {
                self.selected = Selection::Target;
            }
        }
    }
}

impl Tactic {
    fn label(&self) -> &'static str {
        match self {
            Self::Intro => "Intro",
            Self::Assumption { .. } => "Assumption",
            Self::Apply { .. } => "Apply",
            Self::Constructor => "Constructor",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Cases { .. } => "Cases",
            Self::Exists { .. } => "Exists",
            Self::Specialize { .. } => "Specialize",
            Self::Have { .. } => "Have",
            Self::Exfalso => "Exfalso",
            Self::ByContra => "By Contra",
        }
    }
}

/// 選択された仮定が `apply` 可能かを返す。
fn can_apply(formula: &Formula, target: &Formula) -> bool {
    match formula {
        To(_, q) | Iff(_, q) if q.as_ref() == target => true,
        Iff(p, _) if p.as_ref() == target => true,
        Not(_) if target == &False => true,
        False
        | Atom(..)
        | Eq(..)
        | Not(_)
        | And(..)
        | Or(..)
        | All { .. }
        | Ex { .. }
        | To(..)
        | Iff(..) => false,
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
        .replace('→', "->")
        .replace('⇒', "->")
        .replace('⊢', "|-")
        .replace('⊥', "false")
}

/// エラーを JS へ渡す。
fn to_js(e: impl ToString) -> JsValue {
    JsValue::from_str(&e.to_string())
}
