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
    ApplyTactic,
    AssumptionTactic,
    ByContraTactic,
    CasesTactic,
    ConstructorTactic,
    Example,
    ExampleList,
    ExfalsoTactic,
    ExistsTactic,
    FormulaView,
    Game,
    GoalNavView,
    GoalPanelView,
    HomeView,
    HypothesisView,
    IntroTactic,
    LeftTactic,
    MessageView,
    ProofView,
    RightTactic,
    SelectionView,
    SpecializeHypothesisTactic,
    SpecializeTermTactic,
    ToolbarView,
    HaveTactic,
    examples,
};
