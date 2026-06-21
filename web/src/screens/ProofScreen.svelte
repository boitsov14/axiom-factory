<script lang="ts">
  import Formula from "@/components/Formula.svelte";
  import type { ProofScreenProps } from "@/lib/viewProps";
  import { messageClass } from "@/lib/viewHelpers";
  import {
    ApplyTactic,
    AssumptionTactic,
    ByContraTactic,
    CasesTactic,
    ConstructorTactic,
    ExfalsoTactic,
    ExistsTactic,
    HaveTactic,
    IntroTactic,
    LeftTactic,
    RightTactic,
    SpecializeHypothesisTactic,
    SpecializeTermTactic,
    type AppView,
  } from "@/wasm/logic";

  let { app, proof, message, controls }: ProofScreenProps = $props();

  let existsTerm = $state("a");
  let specializeTerm = $state("a");
  let haveFormula = $state("");

  const selectedHypothesis = $derived(proof.selected.hypothesis_index ?? null);

  function run(next: AppView) {
    controls.update(next);
  }

  function applyAssumption(index: number | null) {
    if (index != null) {
      run(AssumptionTactic.apply(app, index));
    }
  }

  function applyHypothesis(index: number | null) {
    if (index != null) {
      run(ApplyTactic.apply(app, index));
    }
  }

  function applyCases(index: number | null) {
    if (index != null) {
      run(CasesTactic.apply(app, index));
    }
  }

  function applySpecializeTerm(index: number | null) {
    if (index != null) {
      run(SpecializeTermTactic.apply(app, index, specializeTerm));
      specializeTerm = "";
    }
  }

  function applySpecializeHypothesis(index: number | null, argumentIndex: number) {
    if (index != null) {
      run(SpecializeHypothesisTactic.apply(app, index, argumentIndex));
    }
  }

  function applyExists() {
    run(ExistsTactic.apply(app, existsTerm));
    existsTerm = "";
  }

  function canAssumption(index: number | null) {
    return index != null && AssumptionTactic.is_enabled(app, index);
  }

  function canApplyHypothesis(index: number | null) {
    return index != null && ApplyTactic.is_enabled(app, index);
  }

  function canCases(index: number | null) {
    return index != null && CasesTactic.is_enabled(app, index);
  }

  function canSpecializeTerm(index: number | null) {
    return index != null && SpecializeTermTactic.is_available(app, index);
  }

  function canApplySpecializeTerm(index: number | null) {
    return index != null && SpecializeTermTactic.can_apply(app, index, specializeTerm);
  }

  function hasSpecializeHypothesisOptions(index: number | null) {
    return index != null && SpecializeHypothesisTactic.has_options(app, index);
  }

  function canSpecializeHypothesis(index: number | null, argumentIndex: number) {
    return index != null && SpecializeHypothesisTactic.is_enabled(app, index, argumentIndex);
  }

  function assumptionDescription(index: number | null) {
    return index == null ? "" : AssumptionTactic.description(app, index);
  }

  function applyDescription(index: number | null) {
    return index == null ? "" : ApplyTactic.description(app, index);
  }

  function casesDescription(index: number | null) {
    return index == null ? "" : CasesTactic.description(app, index);
  }

  function specializeTermDescription(index: number | null) {
    return index == null ? "" : SpecializeTermTactic.description(app, index);
  }

  function specializeHypothesisDescription(index: number | null, argumentIndex: number) {
    return index == null ? "" : SpecializeHypothesisTactic.description(app, index, argumentIndex);
  }

  function applyHave() {
    run(HaveTactic.apply(app, haveFormula));
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
          <h2 class="panel-title">{proof.action_title}</h2>
          <p class="panel-description">{proof.action_hint}</p>
        </div>
        <div class="panel-content space-y-3">
          {#if proof.done}
            <p class="rounded-md border p-3 text-sm text-muted-foreground">No tactics needed.</p>
          {:else if proof.selected.is_target}
            {#if IntroTactic.is_enabled(app)}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{IntroTactic.label()}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{IntroTactic.description(app)}</p>
                  </div>
                  <button class="btn btn-primary shrink-0" type="button" onclick={() => run(IntroTactic.apply(app))}>Apply</button>
                </div>
              </article>
            {/if}

            {#if ConstructorTactic.is_enabled(app)}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{ConstructorTactic.label()}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{ConstructorTactic.description(app)}</p>
                  </div>
                  <button class="btn btn-primary shrink-0" type="button" onclick={() => run(ConstructorTactic.apply(app))}>Apply</button>
                </div>
              </article>
            {/if}

            {#if LeftTactic.is_enabled(app)}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{LeftTactic.label()}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{LeftTactic.description(app)}</p>
                  </div>
                  <button class="btn btn-primary shrink-0" type="button" onclick={() => run(LeftTactic.apply(app))}>Apply</button>
                </div>
              </article>
            {/if}

            {#if RightTactic.is_enabled(app)}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{RightTactic.label()}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{RightTactic.description(app)}</p>
                  </div>
                  <button class="btn btn-primary shrink-0" type="button" onclick={() => run(RightTactic.apply(app))}>Apply</button>
                </div>
              </article>
            {/if}

            {#if ExistsTactic.is_available(app)}
              <article class="rounded-lg border p-4">
                <div>
                  <h3 class="font-semibold">{ExistsTactic.label()}</h3>
                  <p class="mt-1 text-sm text-muted-foreground">{ExistsTactic.description(app)}</p>
                </div>
                <div class="mt-3 flex gap-2">
                  <input class="field flex-1 font-mono" bind:value={existsTerm} placeholder="term" />
                  <button class="btn btn-primary" type="button" disabled={!ExistsTactic.can_apply(app, existsTerm)} onclick={applyExists}>Apply</button>
                </div>
              </article>
            {/if}

            {#if ExfalsoTactic.is_enabled(app)}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{ExfalsoTactic.label()}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{ExfalsoTactic.description(app)}</p>
                  </div>
                  <button class="btn btn-primary shrink-0" type="button" onclick={() => run(ExfalsoTactic.apply(app))}>Apply</button>
                </div>
              </article>
            {/if}

            {#if ByContraTactic.is_enabled(app)}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{ByContraTactic.label()}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{ByContraTactic.description(app)}</p>
                  </div>
                  <button class="btn btn-primary shrink-0" type="button" onclick={() => run(ByContraTactic.apply(app))}>Apply</button>
                </div>
              </article>
            {/if}
          {:else if selectedHypothesis == null}
            <p class="rounded-md border p-3 text-sm text-muted-foreground">Select a hypothesis.</p>
          {:else}
            {#if canAssumption(selectedHypothesis)}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{AssumptionTactic.label()}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{assumptionDescription(selectedHypothesis)}</p>
                  </div>
                  <button class="btn btn-primary shrink-0" type="button" onclick={() => applyAssumption(selectedHypothesis)}>Apply</button>
                </div>
              </article>
            {/if}

            {#if canApplyHypothesis(selectedHypothesis)}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{ApplyTactic.label()}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{applyDescription(selectedHypothesis)}</p>
                  </div>
                  <button class="btn btn-primary shrink-0" type="button" onclick={() => applyHypothesis(selectedHypothesis)}>Apply</button>
                </div>
              </article>
            {/if}

            {#if canCases(selectedHypothesis)}
              <article class="rounded-lg border p-4">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h3 class="font-semibold">{CasesTactic.label()}</h3>
                    <p class="mt-1 text-sm text-muted-foreground">{casesDescription(selectedHypothesis)}</p>
                  </div>
                  <button class="btn btn-primary shrink-0" type="button" onclick={() => applyCases(selectedHypothesis)}>Apply</button>
                </div>
              </article>
            {/if}

            {#if canSpecializeTerm(selectedHypothesis)}
              <article class="rounded-lg border p-4">
                <div>
                  <h3 class="font-semibold">{SpecializeTermTactic.label()}</h3>
                  <p class="mt-1 text-sm text-muted-foreground">{specializeTermDescription(selectedHypothesis)}</p>
                </div>
                <div class="mt-3 flex gap-2">
                  <input class="field flex-1 font-mono" bind:value={specializeTerm} placeholder="term" />
                  <button class="btn btn-primary" type="button" disabled={!canApplySpecializeTerm(selectedHypothesis)} onclick={() => applySpecializeTerm(selectedHypothesis)}>Apply</button>
                </div>
              </article>
            {/if}

            {#if hasSpecializeHypothesisOptions(selectedHypothesis) && proof.goal != null}
              <article class="rounded-lg border p-4">
                <div>
                  <h3 class="font-semibold">{SpecializeHypothesisTactic.label()}</h3>
                  <p class="mt-1 text-sm text-muted-foreground">Instantiate this implication with a matching hypothesis.</p>
                </div>
                <div class="mt-3 grid gap-2">
                  {#each proof.goal.hypotheses as argument (argument.index)}
                    {#if canSpecializeHypothesis(selectedHypothesis, argument.index)}
                      <button class="choice-card text-sm" type="button" onclick={() => applySpecializeHypothesis(selectedHypothesis, argument.index)}>
                        {specializeHypothesisDescription(selectedHypothesis, argument.index)}
                        <span class="mt-1 block font-mono">{argument.formula.copy}</span>
                      </button>
                    {/if}
                  {/each}
                </div>
              </article>
            {/if}
          {/if}
        </div>
      </section>

      <section class="panel">
        <div class="panel-header">
          <h2 class="panel-title">{proof.have_title}</h2>
          <p class="panel-description">{proof.have_hint}</p>
        </div>
        <div class="panel-content space-y-3">
          {#if !HaveTactic.is_available(app)}
            <p class="rounded-md border p-3 text-sm text-muted-foreground">{proof.have_unavailable_text}</p>
          {:else}
            <div>
              <h3 class="font-semibold">{HaveTactic.label()}</h3>
              <p class="mt-1 text-sm text-muted-foreground">{HaveTactic.description(app)}</p>
            </div>
            <div class="flex gap-2">
              <input class="field flex-1 font-mono" bind:value={haveFormula} placeholder={proof.have_placeholder} />
              <button class="btn btn-primary" type="button" disabled={!HaveTactic.can_apply(app, haveFormula)} onclick={applyHave}>{proof.have_add_label}</button>
            </div>
          {/if}
        </div>
      </section>
    </aside>
  </div>
</main>
