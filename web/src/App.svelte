<script lang="ts">
  import { onMount } from "svelte";
  import init, { Game, examples as getExamples, type ExampleList, type ProofView } from "@/wasm/logic";
  import HomeScreen from "@/screens/HomeScreen.svelte";
  import ProofScreen from "@/screens/ProofScreen.svelte";

  let ready = $state(false);
  let game = $state<Game | null>(null);
  let proofView = $state<ProofView | null>(null);
  let startupError = $state("");
  let homeInput = $state("P, P -> Q |- Q");
  let examplesData = $state<ExampleList>({ examples: [] });
  let message = $state<{ text: string; is_error: boolean }>({ text: "", is_error: false });

  onMount(async () => {
    try {
      await init();
      examplesData = getExamples();
      ready = true;
    } catch (e) {
      startupError = e instanceof Error ? e.message : String(e);
      console.error(e);
    }
  });

  function update(delta: ProofView) {
    proofView = delta;
    message = { text: delta.message.text, is_error: delta.message.is_error };
  }

  function startProof(input: string) {
    try {
      const g = new Game(input);
      game = g;
      update(g.proof_view());
    } catch (e) {
      message = { text: e instanceof Error ? e.message : String(e), is_error: true };
    }
  }

  function chooseExample(index: number) {
    const ex = examplesData.examples[index];
    if (ex) startProof(ex.input);
  }

  function openHome() {
    game = null;
    proofView = null;
    message = { text: "", is_error: false };
  }

  function setInput(val: string) {
    homeInput = val;
  }
</script>

{#if startupError.length > 0}
  <main class="app-shell">
    <section class="panel mx-auto max-w-xl p-6">
      <p class="text-sm font-semibold text-red-700">Failed to initialize the proof engine.</p>
      <p class="mt-2 break-words text-sm text-muted-foreground">{startupError}</p>
    </section>
  </main>
{:else if !ready}
  <main class="app-shell">
    <section class="panel mx-auto max-w-xl p-6">
      <p class="text-sm text-muted-foreground">Loading...</p>
    </section>
  </main>
{:else if game == null}
  <HomeScreen
    {homeInput}
    {message}
    examples={examplesData.examples}
    onStartProof={startProof}
    onChooseExample={chooseExample}
    onSetInput={setInput}
  />
{:else if proofView !== null}
  <ProofScreen
    {game}
    proof={proofView}
    {message}
    onUpdate={update}
    onOpenHome={openHome}
  />
{/if}
