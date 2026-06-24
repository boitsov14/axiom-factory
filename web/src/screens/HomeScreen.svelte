<script lang="ts">
  import type { Example } from "@/wasm/logic";

  let {
    homeInput,
    message,
    examples,
    onStartProof,
    onChooseExample,
    onSetInput,
  }: {
    homeInput: string;
    message: { text: string; is_error: boolean };
    examples: Example[];
    onStartProof: (input: string) => void;
    onChooseExample: (index: number) => void;
    onSetInput: (val: string) => void;
  } = $props();
</script>

<main class="app-shell">
  <div class="mx-auto grid max-w-6xl gap-6">
    <header class="space-y-3 py-8">
      <p class="eyebrow">Axiom Factory</p>
      <h1 class="text-4xl font-bold tracking-tight">Build proofs by applying tactics.</h1>
      <p class="max-w-2xl text-muted-foreground">Enter a propositional sequent or choose an example. Rust owns the game state and returns a renderable view model.</p>
    </header>

    <section class="panel">
      <div class="panel-header">
        <h2 class="panel-title">Start a proof</h2>
        <p class="panel-description">Use ASCII syntax such as P, P -> Q |- Q.</p>
      </div>
      <div class="panel-content space-y-4">
        <label class="grid gap-2">
          <span class="text-sm font-medium">Theorem</span>
          <textarea
            class="field min-h-32 resize-y font-mono"
            value={homeInput}
            rows="5"
            placeholder="P, P -> Q |- Q"
            oninput={(event) => onSetInput((event.currentTarget as HTMLTextAreaElement).value)}
          ></textarea>
        </label>
        <div class="flex items-center gap-2">
          <button class="btn btn-primary" type="button" onclick={() => onStartProof(homeInput)}>
            Start proof
          </button>
        </div>
        {#if message.text.length > 0 && message.is_error}
          <p class="message message-error">{message.text}</p>
        {/if}
      </div>
    </section>

    <section class="panel">
      <div class="panel-header">
        <h2 class="panel-title">Examples</h2>
        <p class="panel-description">Choose one to open the proof screen.</p>
      </div>
      <div class="panel-content">
        <div class="grid gap-3 md:grid-cols-2">
          {#each examples as example, index (example.title)}
            <button type="button" class="choice-card" onclick={() => onChooseExample(index)}>
              <span class="block font-medium">{example.title}</span>
              <span class="mt-2 block font-mono text-sm text-muted-foreground">{example.input}</span>
            </button>
          {/each}
        </div>
      </div>
    </section>
  </div>
</main>
