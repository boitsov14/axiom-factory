<script lang="ts">
  import type { HomeScreenProps } from "@/lib/viewProps";
  import { messageClass, shouldShowHomeMessage } from "@/lib/viewHelpers";

  let { home, message, controls }: HomeScreenProps = $props();

  const ready = $derived(controls !== null);
  const showMessage = $derived(shouldShowHomeMessage(message));
</script>

<main class="app-shell">
  <div class="mx-auto grid max-w-6xl gap-6">
    <header class="space-y-3 py-8">
      <p class="eyebrow">{home.title}</p>
      <h1 class="text-4xl font-bold tracking-tight">{home.subtitle}</h1>
      <p class="max-w-2xl text-muted-foreground">{home.description}</p>
    </header>

    <section class="panel">
      <div class="panel-header">
        <h2 class="panel-title">Start a proof</h2>
        <p class="panel-description">{home.syntax_hint}</p>
      </div>
      <div class="panel-content space-y-4">
        <label class="grid gap-2">
          <span class="text-sm font-medium">{home.input_label}</span>
          <textarea
            class="field min-h-32 resize-y font-mono"
            value={home.input}
            rows="5"
            placeholder="P, P -> Q |- Q"
            oninput={(event) => controls?.setInput((event.currentTarget as HTMLTextAreaElement).value)}
          ></textarea>
        </label>
        <div class="flex items-center gap-2">
          <button class="btn btn-primary" type="button" disabled={!ready} onclick={() => controls?.startProof(home.input)}>
            {home.start_label}
          </button>
        </div>
        {#if showMessage}
          <p class={messageClass(message)}>{message.text}</p>
        {/if}
      </div>
    </section>

    <section class="panel">
      <div class="panel-header">
        <h2 class="panel-title">{home.examples_title}</h2>
        <p class="panel-description">{home.examples_hint}</p>
      </div>
      <div class="panel-content">
        <div class="grid gap-3 md:grid-cols-2">
          {#each home.examples as example, index (example.title)}
            <button type="button" class="choice-card" onclick={() => controls?.chooseExample(index)}>
              <span class="block font-medium">{example.title}</span>
              <span class="mt-2 block font-mono text-sm text-muted-foreground">{example.input}</span>
            </button>
          {/each}
        </div>
      </div>
    </section>
  </div>
</main>
