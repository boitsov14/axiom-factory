<script lang="ts">
  import { onMount } from "svelte";
  import init, { App, type AppView } from "@/wasm/logic";
  import { createAppControls, type AppControls } from "@/lib/appClient";
  import HomeScreen from "@/screens/HomeScreen.svelte";
  import ProofScreen from "@/screens/ProofScreen.svelte";

  let app = $state<App | null>(null);
  let controls = $state<AppControls | null>(null);
  let view = $state<AppView | null>(null);
  let startupError = $state("");

  onMount(async () => {
    try {
      await init();
      const nextApp = new App();
      app = nextApp;
      view = nextApp.view();
      controls = createAppControls(nextApp, (next) => {
        view = next;
      });
    } catch (error) {
      startupError = error instanceof Error ? error.message : String(error);
      console.error(error);
    }
  });
</script>

{#if startupError.length > 0}
  <main class="app-shell">
    <section class="panel mx-auto max-w-xl p-6">
      <p class="text-sm font-semibold text-red-700">Failed to initialize the proof engine.</p>
      <p class="mt-2 break-words text-sm text-muted-foreground">{startupError}</p>
    </section>
  </main>
{:else if view === null}
  <main class="app-shell">
    <section class="panel mx-auto max-w-xl p-6">
      <p class="text-sm text-muted-foreground">Loading...</p>
    </section>
  </main>
{:else if view.proof == null}
  <HomeScreen home={view.home} message={view.message} {controls} />
{:else if controls !== null && app !== null}
  <ProofScreen {app} proof={view.proof} message={view.message} {controls} />
{/if}
