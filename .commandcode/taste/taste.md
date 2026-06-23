# rust-testing
- Write `mod tests` at the bottom of the file. Confidence: 0.70
- Avoid `is_ok()`/`is_err()` in tests; verify actual parsed/result values to confirm correctness. Confidence: 0.70

# syntax-design
- Use mathematical/logic textbook notation (e.g., `P(x)` instead of Lean-style `P x`). Confidence: 0.75
- Use right-associative infix operators with precedence: ¬, ∀, ∃ > ∧ > ∨ > → > ↔. Confidence: 0.75
- Format quantifiers as `∀x∀y` (no space-separated variables) and `∀x:N P(x)` (colon syntax, parens around formula). Confidence: 0.75

