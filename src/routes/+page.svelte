<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

  let isBonking = $state(false);
  let bonkCount = $state(0);
  let isHovering = $state(false);
  let timeout: ReturnType<typeof setTimeout> | null = null;
  let hoverTimeout: ReturnType<typeof setTimeout> | null = null;
  let wrapperEl: HTMLDivElement | undefined = $state();
  let unlisten: UnlistenFn | null = null;

  const BONK_DURATION_MS = 150;
  const BONK_COUNT_KEY = "bongo-nat-bonk-count";

  function handleKeydown(e: KeyboardEvent) {
    if (e.repeat) return;
    console.log("[bongo-nat] Browser keydown detected");
    onBonk();
  }

  function handleMousedown(_e: MouseEvent) {
    console.log("[bongo-nat] Browser mousedown detected");
    onBonk();
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
  }

  function onBonk() {
    if (timeout) clearTimeout(timeout);
    isBonking = true;
    bonkCount++;
    saveBonkCount();
    timeout = setTimeout(() => {
      isBonking = false;
    }, BONK_DURATION_MS);
  }

  function saveBonkCount() {
    try {
      localStorage.setItem(BONK_COUNT_KEY, bonkCount.toString());
    } catch {}
  }

  function loadBonkCount() {
    try {
      const stored = localStorage.getItem(BONK_COUNT_KEY);
      if (stored) bonkCount = parseInt(stored, 10) || 0;
    } catch {}
  }

  onMount(async () => {
    console.log("[bongo-nat] Frontend mounted");
    loadBonkCount();

    // Try Tauri event listener (works inside Tauri webview)
    let hasTauri = false;
    try {
      unlisten = await listen("keypress", () => {
        console.log("[bongo-nat] Tauri keypress event received!");
        onBonk();
      });
      hasTauri = true;
      console.log("[bongo-nat] Tauri event listener registered");
    } catch (e) {
      console.warn("[bongo-nat] Tauri API not available, using browser fallback", e);
    }

    // Only use browser listeners as fallback when Tauri is not available
    if (!hasTauri) {
      window.addEventListener("keydown", handleKeydown);
      window.addEventListener("mousedown", handleMousedown);
      console.log("[bongo-nat] Browser keydown/mousedown listeners registered");
    }
    window.addEventListener("contextmenu", handleContextMenu);

    // Track hover via pointermove – more reliable than enter/leave on transparent windows
    document.addEventListener("pointermove", handlePointerMove);
    document.addEventListener("mouseleave", handleDocMouseLeave);
  });

  function handlePointerMove(e: PointerEvent) {
    if (!wrapperEl) return;
    const rect = wrapperEl.getBoundingClientRect();
    const over =
      e.clientX >= rect.left &&
      e.clientX <= rect.right &&
      e.clientY >= rect.top &&
      e.clientY <= rect.bottom;

    isHovering = over;

    // Safety: auto-hide after 2s of no pointer movement
    if (hoverTimeout) clearTimeout(hoverTimeout);
    if (over) {
      hoverTimeout = setTimeout(() => {
        isHovering = false;
      }, 2000);
    }
  }

  function handleDocMouseLeave() {
    isHovering = false;
    if (hoverTimeout) clearTimeout(hoverTimeout);
  }

  onDestroy(() => {
    if (unlisten) unlisten();
    if (timeout) clearTimeout(timeout);
    if (hoverTimeout) clearTimeout(hoverTimeout);
    if (typeof window !== "undefined") {
      window.removeEventListener("keydown", handleKeydown);
      window.removeEventListener("mousedown", handleMousedown);
      window.removeEventListener("contextmenu", handleContextMenu);
      document.removeEventListener("pointermove", handlePointerMove);
      document.removeEventListener("mouseleave", handleDocMouseLeave);
    }
  });
</script>

<div class="nat-container" data-tauri-drag-region>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="nat-wrapper"
    data-tauri-drag-region
    bind:this={wrapperEl}
  >
    {#if isBonking}
      {#if bonkCount % 2 === 0}
        <img src="/nat-bonk-left.svg" alt="Nat bonking left!" class="nat-image" draggable="false" />
      {:else}
        <img src="/nat-bonk-right.svg" alt="Nat bonking right!" class="nat-image" draggable="false" />
      {/if}
    {:else}
      <img src="/nat-idle.svg" alt="Nat idle" class="nat-image" draggable="false" />
    {/if}
    <div class="bonk-tooltip" class:visible={isHovering}>
      {bonkCount.toLocaleString()}
    </div>
  </div>
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
    background: rgba(0, 0, 0, 1);
    box-sizing: border-box;
    border: 5px solid green;
    border-radius: 20px;
    cursor: grab;
  }

  .nat-container:active {
    cursor: grabbing;
  }

  .nat-wrapper {
    position: relative;
    display: inline-block;
  }

  .nat-image {
    width: 280px;
    height: 280px;
    pointer-events: none;
    image-rendering: auto;
  }

  .bonk-tooltip {
    position: absolute;
    bottom: -8px;
    left: 50%;
    transform: translateX(-50%) translateY(-100%);
    background: rgba(30, 30, 30, 0.9);
    color: #fff;
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 13px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 6px;
    white-space: nowrap;
    pointer-events: none;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .bonk-tooltip.visible {
    opacity: 1;
  }
</style>
