<script lang="ts">
  import katex from "katex";
  import type { FormulaProps } from "@/lib/viewProps";
  import { cx } from "@/lib/utils";

  let {
    formula,
    label,
    selected = false,
    selectable = false,
    size = "base",
    onSelect,
  }: FormulaProps = $props();

  let copied = $state(false);

  const html = $derived(
    katex.renderToString(formula.display, {
      displayMode: false,
      throwOnError: false,
      trust: false,
    }),
  );

  async function copyFormula() {
    await navigator.clipboard.writeText(formula.copy);
    copied = true;
    window.setTimeout(() => {
      copied = false;
    }, 900);
  }
</script>

<div class={cx("formula-box", selectable && "formula-selectable", selectable && selected && "formula-selected")}>
  <div class="flex items-start justify-between gap-3">
    {#if selectable}
      <button class="min-w-0 flex-1 text-left" type="button" onclick={() => onSelect?.()}>
        {#if label != null}
          <span class="formula-label">{label}</span>
        {/if}
        <span class={cx("block overflow-x-auto", size === "large" && "text-xl")}>
          {@html html}
        </span>
      </button>
    {:else}
      <div class="min-w-0 flex-1">
        {#if label != null}
          <div class="formula-label">{label}</div>
        {/if}
        <div class={cx("overflow-x-auto", size === "large" && "text-xl")}>
          {@html html}
        </div>
      </div>
    {/if}

    <button class="btn btn-ghost h-8 shrink-0 px-2 opacity-70 hover:opacity-100" type="button" aria-label="Copy formula" title="Copy formula" onclick={copyFormula}>
      <span class="text-xs">{copied ? "Copied" : "Copy"}</span>
    </button>
  </div>
</div>
