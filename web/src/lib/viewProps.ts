import type { FormulaView } from "@/wasm/logic";

export interface FormulaProps {
  formula: FormulaView;
  label?: string;
  selected?: boolean;
  selectable?: boolean;
  size?: "base" | "large";
  onSelect?: () => void;
}
