mod display;
mod game;
mod ids;
mod open;
mod parser;
mod syntax;
mod tactic;

pub use game::{
    AvailableTactics,
    Example,
    ExampleList,
    FormulaView,
    Game,
    GoalNavView,
    GoalPanelView,
    HypothesisTacticView,
    HypothesisTactics,
    HypothesisView,
    MessageView,
    ProofView,
    SelectionView,
    TacticView,
    ToolbarView,
    examples,
};
