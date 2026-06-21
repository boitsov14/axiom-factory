mod display;
mod game;
mod open;
mod parser;
mod proof;
mod syntax;
mod tactic;

pub use game::{
    App,
    AppView,
    ArgumentOptionView,
    Example,
    ExampleList,
    FormulaView,
    Game,
    GoalNavView,
    GoalPanelView,
    HavePanelView,
    HomeView,
    HypothesisView,
    MessageView,
    ProofView,
    SelectionView,
    TacticButtonView,
    TacticCommandTemplateView,
    TacticPanelView,
    TacticTextInputView,
    ToolbarView,
    examples,
};
