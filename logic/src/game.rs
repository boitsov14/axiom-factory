use crate::{
    parser,
    proof::Goal,
    syntax::{Formula, Formula::*, Term},
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
    pub tactics_panel: TacticPanelView,
    pub have_panel: HavePanelView,
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
pub struct TacticPanelView {
    pub title: String,
    pub hint: String,
    pub empty_text: String,
    pub tactics: Vec<TacticButtonView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct HavePanelView {
    pub title: String,
    pub hint: String,
    pub placeholder: String,
    pub add_label: String,
    pub unavailable_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub tactic: Option<TacticButtonView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct TacticButtonView {
    pub label: String,
    pub description: String,
    pub apply_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub command: Option<TacticCommandTemplateView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub text_input: Option<TacticTextInputView>,
    pub argument_options: Vec<ArgumentOptionView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct TacticTextInputView {
    pub placeholder: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct ArgumentOptionView {
    pub label: String,
    pub formula: FormulaView,
    pub command: TacticCommandTemplateView,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[serde(tag = "type", rename_all = "PascalCase")]
#[tsify(into_wasm_abi)]
pub enum TacticCommandTemplateView {
    Intro,
    Assumption {
        hypothesis_index: usize,
    },
    Apply {
        hypothesis_index: usize,
    },
    Constructor,
    Left,
    Right,
    Cases {
        hypothesis_index: usize,
    },
    Exists,
    SpecializeTerm {
        hypothesis_index: usize,
    },
    SpecializeHypothesis {
        hypothesis_index: usize,
        argument_index: usize,
    },
    Have,
    Exfalso,
    ByContra,
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
                input: "all x, P(x) |- P(a)".into(),
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
        let (hypotheses, target) = parser::parse_goal(&normalized).map_err(|e| e.to_string())?;
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
            tactics_panel: self.tactics_panel_view(),
            have_panel: self.have_panel_view(),
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

    fn tactics_panel_view(&self) -> TacticPanelView {
        let tactics = self.selected_tactics();
        TacticPanelView {
            title: self.selection_view().label,
            hint: "These actions apply to the selected formula.".into(),
            empty_text: if self.goals.is_empty() {
                "No tactics needed.".into()
            } else {
                "No applicable tactics for this selection.".into()
            },
            tactics,
        }
    }

    fn have_panel_view(&self) -> HavePanelView {
        HavePanelView {
            title: "Have".into(),
            hint: "Introduce a separate intermediate claim.".into(),
            placeholder: "formula".into(),
            add_label: "Add".into(),
            unavailable_text: "Have is not available.".into(),
            tactic: (!self.goals.is_empty())
                .then(|| text_tactic_button(TacticCommandTemplateView::Have, "formula")),
        }
    }

    fn selected_tactics(&self) -> Vec<TacticButtonView> {
        let Some(goal) = self.current_goal() else {
            return vec![];
        };
        match self.selected {
            Selection::Target => target_tactics(goal),
            Selection::Hypothesis(index) => hypothesis_tactics(goal, index),
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

/// ターゲットに直接適用できるタクティクを返す。
fn target_tactics(goal: &Goal) -> Vec<TacticButtonView> {
    let mut tactics = vec![];
    match &goal.target {
        To(..) | All { .. } | Not(_) => {
            tactics.push(tactic_button(TacticCommandTemplateView::Intro));
        }
        And(..) | Iff(..) => {
            tactics.push(tactic_button(TacticCommandTemplateView::Constructor));
        }
        Or(..) => {
            tactics.push(tactic_button(TacticCommandTemplateView::Left));
            tactics.push(tactic_button(TacticCommandTemplateView::Right));
        }
        Ex { .. } => {
            tactics.push(text_tactic_button(
                TacticCommandTemplateView::Exists,
                "term",
            ));
        }
        False | Atom(..) | Eq(..) => {}
    }

    if goal.target != False {
        tactics.push(tactic_button(TacticCommandTemplateView::Exfalso));
        tactics.push(tactic_button(TacticCommandTemplateView::ByContra));
    }

    tactics
}

/// 選択された仮定に適用できるタクティクを返す。
fn hypothesis_tactics(goal: &Goal, hyp: usize) -> Vec<TacticButtonView> {
    let Some(formula) = goal.hypotheses.get(hyp) else {
        return vec![];
    };

    let mut tactics = vec![];
    if formula == &goal.target {
        tactics.push(tactic_button(TacticCommandTemplateView::Assumption {
            hypothesis_index: hyp,
        }));
    }
    if can_apply(formula, &goal.target) {
        tactics.push(tactic_button(TacticCommandTemplateView::Apply {
            hypothesis_index: hyp,
        }));
    }
    match formula {
        And(..) | Or(..) | Iff(..) | Ex { .. } | False => {
            tactics.push(tactic_button(TacticCommandTemplateView::Cases {
                hypothesis_index: hyp,
            }));
        }
        All { .. } => {
            tactics.push(text_tactic_button(
                TacticCommandTemplateView::SpecializeTerm {
                    hypothesis_index: hyp,
                },
                "term",
            ));
        }
        To(p, _) => {
            let argument_options = matching_hypotheses(goal, p)
                .into_iter()
                .filter_map(|index| {
                    goal.hypotheses
                        .get(index)
                        .map(|formula| ArgumentOptionView {
                            label: format!("Use hypothesis {}", index + 1),
                            formula: formula_view(formula),
                            command: TacticCommandTemplateView::SpecializeHypothesis {
                                hypothesis_index: hyp,
                                argument_index: index,
                            },
                        })
                })
                .collect::<Vec<_>>();
            if !argument_options.is_empty() {
                tactics.push(argument_tactic_button(
                    TacticCommandTemplateView::SpecializeHypothesis {
                        hypothesis_index: hyp,
                        argument_index: 0,
                    },
                    argument_options,
                ));
            }
        }
        Atom(..) | Eq(..) | Not(_) => {}
    }

    tactics
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

/// 指定式に一致する仮定番号を返す。
fn matching_hypotheses(goal: &Goal, formula: &Formula) -> Vec<usize> {
    goal.hypotheses
        .iter()
        .enumerate()
        .filter_map(|(i, p)| (p == formula).then_some(i))
        .collect()
}

/// タクティク候補を表示用に作る。
fn tactic_button(command: TacticCommandTemplateView) -> TacticButtonView {
    let label = tactic_label(&command);
    TacticButtonView {
        label: label.into(),
        description: tactic_description(&command).into(),
        apply_label: tactic_apply_label(&command).into(),
        command: Some(command),
        text_input: None,
        argument_options: vec![],
    }
}

fn text_tactic_button(command: TacticCommandTemplateView, placeholder: &str) -> TacticButtonView {
    let label = tactic_label(&command);
    TacticButtonView {
        label: label.into(),
        description: tactic_description(&command).into(),
        apply_label: tactic_apply_label(&command).into(),
        command: Some(command),
        text_input: Some(TacticTextInputView {
            placeholder: placeholder.into(),
        }),
        argument_options: vec![],
    }
}

fn argument_tactic_button(
    command: TacticCommandTemplateView,
    argument_options: Vec<ArgumentOptionView>,
) -> TacticButtonView {
    let label = tactic_label(&command);
    TacticButtonView {
        label: label.into(),
        description: tactic_description(&command).into(),
        apply_label: tactic_apply_label(&command).into(),
        command: None,
        text_input: None,
        argument_options,
    }
}

fn tactic_apply_label(command: &TacticCommandTemplateView) -> &'static str {
    match command {
        TacticCommandTemplateView::Have => "Add",
        TacticCommandTemplateView::Intro
        | TacticCommandTemplateView::Assumption { .. }
        | TacticCommandTemplateView::Apply { .. }
        | TacticCommandTemplateView::Constructor
        | TacticCommandTemplateView::Left
        | TacticCommandTemplateView::Right
        | TacticCommandTemplateView::Cases { .. }
        | TacticCommandTemplateView::Exists
        | TacticCommandTemplateView::SpecializeTerm { .. }
        | TacticCommandTemplateView::SpecializeHypothesis { .. }
        | TacticCommandTemplateView::Exfalso
        | TacticCommandTemplateView::ByContra => "Apply",
    }
}

fn tactic_label(command: &TacticCommandTemplateView) -> &'static str {
    match command {
        TacticCommandTemplateView::Intro => "Intro",
        TacticCommandTemplateView::Assumption { .. } => "Assumption",
        TacticCommandTemplateView::Apply { .. } => "Apply",
        TacticCommandTemplateView::Constructor => "Constructor",
        TacticCommandTemplateView::Left => "Left",
        TacticCommandTemplateView::Right => "Right",
        TacticCommandTemplateView::Cases { .. } => "Cases",
        TacticCommandTemplateView::Exists => "Exists",
        TacticCommandTemplateView::SpecializeTerm { .. }
        | TacticCommandTemplateView::SpecializeHypothesis { .. } => "Specialize",
        TacticCommandTemplateView::Have => "Have",
        TacticCommandTemplateView::Exfalso => "Exfalso",
        TacticCommandTemplateView::ByContra => "By Contra",
    }
}

fn tactic_description(command: &TacticCommandTemplateView) -> &'static str {
    match command {
        TacticCommandTemplateView::Intro => {
            "Introduce an implication, universal quantifier, or negation."
        }
        TacticCommandTemplateView::Assumption { .. } => {
            "Close the goal with this matching hypothesis."
        }
        TacticCommandTemplateView::Apply { .. } => {
            "Apply this implication, negation, or equivalence hypothesis to the target."
        }
        TacticCommandTemplateView::Constructor => {
            "Split a conjunction or equivalence target into subgoals."
        }
        TacticCommandTemplateView::Left => "Prove the left side of a disjunction target.",
        TacticCommandTemplateView::Right => "Prove the right side of a disjunction target.",
        TacticCommandTemplateView::Cases { .. } => "Split this hypothesis or eliminate falsehood.",
        TacticCommandTemplateView::Exists => "Provide a witness for the existential target.",
        TacticCommandTemplateView::SpecializeTerm { .. }
        | TacticCommandTemplateView::SpecializeHypothesis { .. } => {
            "Instantiate this universal or implication hypothesis."
        }
        TacticCommandTemplateView::Have => {
            "Create a separate subgoal and add it as a later hypothesis."
        }
        TacticCommandTemplateView::Exfalso => "Reduce the current target to False.",
        TacticCommandTemplateView::ByContra => "Prove the target by contradiction.",
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
        .replace('⊤', "true")
}

/// エラーを JS へ渡す。
fn to_js(e: impl ToString) -> JsValue {
    JsValue::from_str(&e.to_string())
}
