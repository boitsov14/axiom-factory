<script lang="ts">
  import Formula from "@/components/Formula.svelte";
  import type { Game, ProofView } from "@/wasm/logic";
  import { messageClass } from "@/lib/viewHelpers";

  let {
    game,
    proof,
    message,
    onUpdate,
    onOpenHome,
  }: {
    game: Game;
    proof: ProofView;
    message: { text: string; is_error: boolean };
    onUpdate: (next: ProofView) => void;
    onOpenHome: () => void;
  } = $props();

  let existsTerm = $state("");
  let specializeTerm = $state("");
  let haveFormula = $state("");

  const messageView = $derived({
    text: message.text,
    visible: message.text.length > 0,
    is_info: !message.is_error,
    is_success: !message.is_error,
    is_error: message.is_error,
  });

  function run(fn: () => ProofView) {
    onUpdate(fn());
  }

  function applyTargetTactic(tactic: { label: string; needs_term_input: boolean }) {
    if (tactic.needs_term_input) return;
    switch (tactic.label) {
      case "Assumption":
        run(() => game.apply_assumption());
        break;
      case "Intro¬": case "Intro→": case "Intro∀":
        run(() => game.apply_intro());
        break;
      case "Conj∧": case "Conj↔":
        run(() => game.apply_constructor());
        break;
      case "Left":
        run(() => game.apply_left());
        break;
      case "Right":
        run(() => game.apply_right());
        break;
      case "ExFalso":
        run(() => game.apply_exfalso());
        break;
      case "ByContra":
        run(() => game.apply_by_contra());
        break;
    }
  }

  function applyHypothesisTactic(tacticKind: string, hypIndex: number, argIndex?: number) {
    switch (tacticKind) {
      case "apply":
        run(() => game.apply_hypothesis(hypIndex));
        break;
      case "cases":
        run(() => game.apply_cases(hypIndex));
        break;
      case "specialize_term":
        run(() => game.apply_specialize_with_term(hypIndex, specializeTerm));
        specializeTerm = "";
        break;
      case "specialize_hypothesis":
        if (argIndex != null) {
          run(() => game.apply_specialize_with_hypothesis(hypIndex, argIndex));
        }
        break;
    }
  }

  function applyExists() {
    run(() => game.apply_exists(existsTerm));
    existsTerm = "";
  }

  function applyHave() {
    run(() => game.apply_have(haveFormula));
    haveFormula = "";
  }
</script>

<main class="app-shell">
  <div class="mx-auto grid max-w-[1500px] gap-6 xl:grid-cols-[1fr_420px]">
    <section class="space-y-6">
      <section class="panel">
        <div class="flex flex-wrap items-center justify-between gap-3 p-4">
          <div>
            <div class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{proof.goal_nav.label}</div>
            <div class="mt-1 font-mono text-sm text-muted-foreground">{proof.theorem_input}</div>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <button class="btn btn-outline" type="button" onclick={onOpenHome}>{proof.toolbar.home_label}</button>
            <button class="btn btn-secondary" type="button" disabled={!proof.toolbar.can_undo} onclick={() => run(() => game.undo())}>{proof.toolbar.undo_label}</button>
            <button class="btn btn-secondary" type="button" disabled={!proof.toolbar.can_redo} onclick={() => run(() => game.redo())}>{proof.toolbar.redo_label}</button>
          </div>
        </div>
      </section>

      {#if messageView.visible}
        <p class={messageClass(messageView)}>{message.text}</p>
      {/if}

      {#if proof.goal == null}
        <section class="panel p-10 text-center">
          <h1 class="text-3xl font-bold">{proof.title}</h1>
          <p class="mt-2 text-muted-foreground">{proof.subtitle}</p>
        </section>
      {:else}
        {@const goal = proof.goal}
        <section class="panel">
          <div class="panel-header">
            <div class="flex flex-wrap items-center justify-between gap-3">
              <div>
                <h2 class="panel-title">{goal.title}</h2>
                <p class="panel-description">{goal.hint}</p>
              </div>
              <div class="flex items-center gap-2">
                <button class="btn btn-outline" type="button" disabled={!proof.goal_nav.can_previous} aria-label={proof.goal_nav.previous_label} onclick={() => run(() => game.previous_goal())}>←</button>
                <span class="text-sm text-muted-foreground">{proof.goal_nav.current} / {proof.goal_nav.total}</span>
                <button class="btn btn-outline" type="button" disabled={!proof.goal_nav.can_next} aria-label={proof.goal_nav.next_label} onclick={() => run(() => game.next_goal())}>→</button>
              </div>
            </div>
          </div>
          <div class="panel-content grid gap-5 lg:grid-cols-[1fr_1.1fr]">
            <section class="space-y-3">
              <h3 class="section-label">{goal.hypotheses_title}</h3>
              {#if goal.hypotheses.length === 0}
                <p class="rounded-md border p-3 text-sm text-muted-foreground">{goal.no_hypotheses_text}</p>
              {:else}
                <div class="grid gap-3">
                  {#each goal.hypotheses as hypothesis (hypothesis.index)}
                    <Formula
                      formula={hypothesis.formula}
                      label={hypothesis.label}
                      selected={hypothesis.selected}
                      selectable
                      onSelect={() => run(() => game.select_hypothesis(hypothesis.index))}
                    />
                  {/each}
                </div>
              {/if}
            </section>

            <section>
              <Formula
                formula={goal.target}
                label={goal.target_title}
                selected={proof.selected.is_target}
                selectable
                size="large"
                onSelect={() => run(() => game.select_target())}
              />
            </section>
          </div>
        </section>
      {/if}
    </section>

    <aside class="space-y-6">
      <section class="panel">
        <div class="panel-header">
          <h2 class="panel-title">{proof.selected.label}</h2>
          <p class="panel-description">Select the target or a hypothesis and apply a tactic.</p>
        </div>
        <div class="panel-content space-y-3">
          {#if proof.done}
            <p class="rounded-md border p-3 text-sm text-muted-foreground">No tactics needed.</p>
          {:else if proof.selected.is_target}
            {#each proof.available_tactics.target as tactic}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{tactic.label}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{tactic.description}</p>
                    <div class="mt-2 space-y-0.5 font-mono text-xs text-muted-foreground">
                      <div class="text-blue-500">{tactic.before}</div>
                      <div class="text-green-600">→ {tactic.after.replace(/\n/g, "\n   ")}</div>
                    </div>
                  </div>
                  {#if tactic.needs_term_input}
                    <div class="shrink-0 self-start">
                      <input class="field w-20 font-mono" bind:value={existsTerm} placeholder="term" />
                    </div>
                  {:else}
                    <button class="btn btn-primary shrink-0" type="button" onclick={() => applyTargetTactic(tactic)}>Apply</button>
                  {/if}
                </div>
                {#if tactic.needs_term_input}
                  <div class="mt-3 flex justify-end">
                    <button class="btn btn-primary" type="button" disabled={existsTerm.trim() === ""} onclick={applyExists}>Apply</button>
                  </div>
                {/if}
              </article>
            {/each}
          {:else}
            {#each proof.available_tactics.hypotheses as ht (ht.hypothesis_index)}
              <div class="space-y-3">
                <p class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                  Hypothesis {ht.hypothesis_index + 1}
                </p>
                {#each ht.tactics as tactic}
                  <article class="rounded-lg border p-4">
                    <div class="flex items-start justify-between gap-3">
                      <div>
                        <h3 class="font-semibold">{tactic.label}</h3>
                        <p class="mt-1 text-sm text-muted-foreground">{tactic.description}</p>
                        <div class="mt-2 space-y-0.5 font-mono text-xs text-muted-foreground">
                          <div class="text-blue-500">{tactic.before}</div>
                          <div class="text-green-600">→ {tactic.after.replace(/\n/g, "\n   ")}</div>
                        </div>
                      </div>
                      <div class="shrink-0 self-start">
                        {#if tactic.kind === "specialize_term"}
                          <input class="field w-20 font-mono" bind:value={specializeTerm} placeholder="term" />
                          <button class="btn btn-primary mt-2" type="button" disabled={specializeTerm.trim() === ""}
                            onclick={() => applyHypothesisTactic(tactic.kind, ht.hypothesis_index)}>Apply</button>
                        {:else if tactic.kind === "specialize_hypothesis"}
                          <button class="btn btn-primary" type="button"
                            onclick={() => applyHypothesisTactic(tactic.kind, ht.hypothesis_index, 0)}>Apply</button>
                        {:else}
                          <button class="btn btn-primary" type="button"
                            onclick={() => applyHypothesisTactic(tactic.kind, ht.hypothesis_index)}>Apply</button>
                        {/if}
                      </div>
                    </div>
                  </article>
                {/each}
              </div>
            {/each}

            {#if proof.available_tactics.hypotheses.length === 0}
              <p class="rounded-md border p-3 text-sm text-muted-foreground">No applicable tactics for the selected hypothesis.</p>
            {/if}
          {/if}
        </div>
      </section>

      <section class="panel">
        <div class="panel-header">
          <h2 class="panel-title">Have</h2>
          <p class="panel-description">Introduce a separate intermediate claim.</p>
        </div>
        <div class="panel-content space-y-3">
          {#if proof.goal == null}
            <p class="rounded-md border p-3 text-sm text-muted-foreground">Have is not available.</p>
          {:else}
            <div>
              <h3 class="font-semibold">Have</h3>
              <p class="mt-1 text-sm text-muted-foreground">Introduce a separate intermediate claim.</p>
            </div>
            <div class="flex gap-2">
              <input class="field flex-1 font-mono" bind:value={haveFormula} placeholder="formula" />
              <button class="btn btn-primary" type="button" disabled={haveFormula.trim() === ""} onclick={applyHave}>Add</button>
            </div>
          {/if}
        </div>
      </section>
    </aside>
  </div>
</main>
