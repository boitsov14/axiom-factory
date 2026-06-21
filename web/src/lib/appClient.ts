import type { App, AppView } from "@/wasm/logic";

export interface AppControls {
  update(view: AppView): void;
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
}

export function createAppControls(app: App, setView: (view: AppView) => void): AppControls {
  const update = (view: AppView) => setView(view);

  return {
    update,
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
  };
}
