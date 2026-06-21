<script lang="ts">
  import Formula from "@/components/Formula.svelte";
  import type { ProofScreenProps } from "@/lib/viewProps";
  import { messageClass } from "@/lib/viewHelpers";

  let { proof, message, controls }: ProofScreenProps = $props();

  let tacticInputs = $state<Record<number, string>>({});
  let haveFormula = $state("");

  function tacticInput(index: number): string {
    return tacticInputs[index] ?? "";
  }

  function updateTacticInput(index: number, event: Event) {
    tacticInputs[index] = (event.currentTarget as HTMLInputElement).value;
  }

  function updateHaveFormula(event: Event) {
    haveFormula = (event.currentTarget as HTMLInputElement).value;
  }

  function applyTextTactic(index: number) {
    const tactic = proof.tactics_panel.tactics[index];
    const input = tacticInput(index).trim();
    if (tactic == null || input.length === 0) {
      return;
    }
    if (controls.applyTactic(tactic, input)) {
      tacticInputs[index] = "";
    }
  }

  function applyHave() {
    const tactic = proof.have_panel.tactic;
    const formula = haveFormula.trim();
    if (tactic == null || formula.length === 0) {
      return;
    }
    if (controls.applyTactic(tactic, formula)) {
      haveFormula = "";
    }
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
            <button class="btn btn-outline" type="button" onclick={controls.openHome}>{proof.toolbar.home_label}</button>
            <button class="btn btn-secondary" type="button" disabled={!proof.toolbar.can_undo} onclick={controls.undo}>{proof.toolbar.undo_label}</button>
            <button class="btn btn-secondary" type="button" disabled={!proof.toolbar.can_redo} onclick={controls.redo}>{proof.toolbar.redo_label}</button>
          </div>
        </div>
      </section>

      {#if message.visible}
        <p class={messageClass(message)}>{message.text}</p>
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
                <button class="btn btn-outline" type="button" disabled={!proof.goal_nav.can_previous} aria-label={proof.goal_nav.previous_label} onclick={controls.previousGoal}>←</button>
                <span class="text-sm text-muted-foreground">{proof.goal_nav.current} / {proof.goal_nav.total}</span>
                <button class="btn btn-outline" type="button" disabled={!proof.goal_nav.can_next} aria-label={proof.goal_nav.next_label} onclick={controls.nextGoal}>→</button>
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
                      onSelect={() => controls.selectHypothesis(hypothesis.index)}
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
                onSelect={controls.selectTarget}
              />
            </section>
          </div>
        </section>
      {/if}
    </section>

    <aside class="space-y-6">
      <section class="panel">
        <div class="panel-header">
          <h2 class="panel-title">{proof.tactics_panel.title}</h2>
          <p class="panel-description">{proof.tactics_panel.hint}</p>
        </div>
        <div class="panel-content space-y-3">
          {#if proof.tactics_panel.tactics.length === 0}
            <p class="rounded-md border p-3 text-sm text-muted-foreground">{proof.tactics_panel.empty_text}</p>
          {:else}
            {#each proof.tactics_panel.tactics as tactic, index (tactic.label + index)}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{tactic.label}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{tactic.description}</p>
                  </div>
                  {#if tactic.text_input == null && tactic.argument_options.length === 0}
                    <button class="btn btn-primary shrink-0" type="button" onclick={() => controls.applyTactic(tactic)}>{tactic.apply_label}</button>
                  {/if}
                </div>

                {#if tactic.text_input != null}
                  <div class="mt-3 flex gap-2">
                    <input
                      class="field flex-1 font-mono"
                      value={tacticInput(index)}
                      placeholder={tactic.text_input.placeholder}
                      oninput={(event) => updateTacticInput(index, event)}
                    />
                    <button class="btn btn-primary" type="button" onclick={() => applyTextTactic(index)}>{tactic.apply_label}</button>
                  </div>
                {/if}

                {#if tactic.argument_options.length > 0}
                  <div class="mt-3 grid gap-2">
                    {#each tactic.argument_options as option (option.label)}
                      <button class="choice-card text-sm" type="button" onclick={() => controls.applyArgument(option)}>
                        {option.label}: <span class="font-mono">{option.formula.copy}</span>
                      </button>
                    {/each}
                  </div>
                {/if}
              </article>
            {/each}
          {/if}
        </div>
      </section>

      <section class="panel">
        <div class="panel-header">
          <h2 class="panel-title">{proof.have_panel.title}</h2>
          <p class="panel-description">{proof.have_panel.hint}</p>
        </div>
        <div class="panel-content space-y-3">
          {#if proof.have_panel.tactic == null}
            <p class="rounded-md border p-3 text-sm text-muted-foreground">{proof.have_panel.unavailable_text}</p>
          {:else}
            {@const tactic = proof.have_panel.tactic}
            <div>
              <h3 class="font-semibold">{tactic.label}</h3>
              <p class="mt-1 text-sm text-muted-foreground">{tactic.description}</p>
            </div>
            <div class="flex gap-2">
              <input class="field flex-1 font-mono" value={haveFormula} placeholder={proof.have_panel.placeholder} oninput={updateHaveFormula} />
              <button class="btn btn-primary" type="button" onclick={applyHave}>{proof.have_panel.add_label}</button>
            </div>
          {/if}
        </div>
      </section>
    </aside>
  </div>
</main>
