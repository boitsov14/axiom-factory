import type { AppControls } from "@/lib/appClient";
import type { App, FormulaView, HomeView, MessageView, ProofView } from "@/wasm/logic";

export interface FormulaProps {
  formula: FormulaView;
  label?: string;
  selected?: boolean;
  selectable?: boolean;
  size?: "base" | "large";
  onSelect?: () => void;
}

export interface HomeScreenProps {
  home: HomeView;
  message: MessageView;
  controls: AppControls | null;
}

export interface ProofScreenProps {
  app: App;
  proof: ProofView;
  message: MessageView;
  controls: AppControls;
}
