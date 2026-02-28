<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

  let isBonking = $state(false);
  let timeout: ReturnType<typeof setTimeout> | null = null;
  let unlisten: UnlistenFn | null = null;

  const BONK_DURATION_MS = 150;

  function handleKeydown(_e: KeyboardEvent) {
    console.log("[bongo-nat] Browser keydown detected");
    onBonk();
  }

  function onBonk() {
    if (timeout) clearTimeout(timeout);
    isBonking = true;
    timeout = setTimeout(() => {
      isBonking = false;
    }, BONK_DURATION_MS);
  }

  onMount(async () => {
    console.log("[bongo-nat] Frontend mounted");

    // Try Tauri event listener (works inside Tauri webview)
    try {
      unlisten = await listen("keypress", () => {
        console.log("[bongo-nat] Tauri keypress event received!");
        onBonk();
      });
      console.log("[bongo-nat] Tauri event listener registered");
    } catch (e) {
      console.warn("[bongo-nat] Tauri API not available, using browser fallback", e);
    }

    // Also listen for native browser keydown as a fallback / debug aid
    window.addEventListener("keydown", handleKeydown);
    console.log("[bongo-nat] Browser keydown listener registered");
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    if (timeout) clearTimeout(timeout);
    if (typeof window !== "undefined") {
      window.removeEventListener("keydown", handleKeydown);
    }
  });
</script>

<div class="nat-container" data-tauri-drag-region>
  {#if isBonking}
    <img src="/nat-bonk.svg" alt="Nat bonking!" class="nat-image" draggable="false" />
  {:else}
    <img src="/nat-idle.svg" alt="Nat idle" class="nat-image" draggable="false" />
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent !important;
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
  }

  .nat-container {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0);
    border-radius: 20px;
    cursor: grab;
  }

  .nat-container:active {
    cursor: grabbing;
  }

  .nat-image {
    width: 280px;
    height: 280px;
    pointer-events: none;
    image-rendering: auto;
  }
</style>
