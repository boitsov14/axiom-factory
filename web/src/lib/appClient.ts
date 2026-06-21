import type {
  App,
  AppView,
  ArgumentOptionView,
  TacticButtonView,
  TacticCommandTemplateView,
} from "@/wasm/logic";

export interface AppControls {
  setInput(input: string): void;
  startProof(input: string): void;
  chooseExample(index: number): void;
  openHome(): void;
  selectTarget(): void;
  selectHypothesis(index: number): void;
  undo(): void;
  redo(): void;
  previousGoal(): void;
  nextGoal(): void;
  applyTactic(tactic: TacticButtonView, input?: string): boolean;
  applyArgument(option: ArgumentOptionView): void;
}

export function createAppControls(app: App, setView: (view: AppView) => void): AppControls {
  const update = (view: AppView) => setView(view);

  return {
    setInput: (input) => update(app.set_input(input)),
    startProof: (input) => update(app.start_proof(input)),
    chooseExample: (index) => update(app.choose_example(index)),
    openHome: () => update(app.open_home()),
    selectTarget: () => update(app.select_target()),
    selectHypothesis: (index) => update(app.select_hypothesis(index)),
    undo: () => update(app.undo()),
    redo: () => update(app.redo()),
    previousGoal: () => update(app.previous_goal()),
    nextGoal: () => update(app.next_goal()),
    applyTactic: (tactic, input) => {
      if (tactic.command == null) {
        return false;
      }
      const view = applyTemplate(app, tactic.command, input);
      if (view == null) {
        return false;
      }
      update(view);
      return true;
    },
    applyArgument: (option) => {
      const view = applyTemplate(app, option.command);
      if (view != null) {
        update(view);
      }
    },
  };
}

function applyTemplate(
  app: App,
  template: TacticCommandTemplateView,
  input?: string,
): AppView | undefined {
  switch (template.type) {
    case "Intro":
      return app.apply_intro();
    case "Assumption":
      return app.apply_assumption(template.hypothesis_index);
    case "Apply":
      return app.apply_hypothesis(template.hypothesis_index);
    case "Constructor":
      return app.apply_constructor();
    case "Left":
      return app.apply_left();
    case "Right":
      return app.apply_right();
    case "Cases":
      return app.apply_cases(template.hypothesis_index);
    case "Exists":
      return input == null ? undefined : app.apply_exists(input);
    case "SpecializeTerm":
      return input == null
        ? undefined
        : app.apply_specialize_with_term(template.hypothesis_index, input);
    case "SpecializeHypothesis":
      return app.apply_specialize_with_hypothesis(
        template.hypothesis_index,
        template.argument_index,
      );
    case "Have":
      return input == null ? undefined : app.apply_have(input);
    case "Exfalso":
      return app.apply_exfalso();
    case "ByContra":
      return app.apply_by_contra();
    default:
      return exhaustive(template);
  }
}

function exhaustive(value: never): never {
  throw new Error(`Unhandled tactic command: ${JSON.stringify(value)}`);
}
