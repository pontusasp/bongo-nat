<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";

  interface BongoVersion {
    name: string;
    path: string | null;
  }

  let isBonking = $state(false);
  let bonkCount = $state(0);
  let isHovering = $state(false);
  let showBackground = $state(false);
  let scalePercent = $state(100);
  let isReady = $state(false);
  let timeout: ReturnType<typeof setTimeout> | null = null;
  let hoverTimeout: ReturnType<typeof setTimeout> | null = null;
  let wrapperEl: HTMLDivElement | undefined = $state();
  let unlisten: UnlistenFn | null = null;
  let unlistenBg: UnlistenFn | null = null;
  let unlistenVersion: UnlistenFn | null = null;
  let unlistenScale: UnlistenFn | null = null;

  // --- Image URLs (reactive) ---
  const DEFAULT_IDLE = "/bongo-nat-idle.png";
  const DEFAULT_LEFT = "/bongo-nat-left.png";
  const DEFAULT_RIGHT = "/bongo-nat-right.png";

  let imgIdle = $state(DEFAULT_IDLE);
  let imgLeft = $state(DEFAULT_LEFT);
  let imgRight = $state(DEFAULT_RIGHT);

  const BONK_DURATION_MS = 150;
  const BONK_COUNT_KEY = "bongo-nat-bonk-count";

  // --- Version loading ---
  async function loadVersionImages(version: BongoVersion) {
    if (!version.path) {
      // Built-in default
      imgIdle = DEFAULT_IDLE;
      imgLeft = DEFAULT_LEFT;
      imgRight = DEFAULT_RIGHT;
      return;
    }

    try {
      const [idle, left, right] = await Promise.all([
        invoke<string>("get_version_image", { versionName: version.name, imageName: "idle" }),
        invoke<string>("get_version_image", { versionName: version.name, imageName: "left" }),
        invoke<string>("get_version_image", { versionName: version.name, imageName: "right" }),
      ]);
      imgIdle = idle;
      imgLeft = left;
      imgRight = right;
      console.log(`[bongo-nat] Loaded images for version: ${version.name}`);
    } catch (e) {
      console.error("[bongo-nat] Failed to load version images, using defaults", e);
      imgIdle = DEFAULT_IDLE;
      imgLeft = DEFAULT_LEFT;
      imgRight = DEFAULT_RIGHT;
    }
  }

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

    // Load persisted background state
    try {
      showBackground = await invoke<boolean>("get_show_background");
    } catch (e) {
      console.warn("[bongo-nat] Could not load background state", e);
    }

    // Load persisted scale percent
    try {
      scalePercent = await invoke<number>("get_scale_percent");
    } catch (e) {
      console.warn("[bongo-nat] Could not load scale percent", e);
    }

    // Load persisted bongo version
    try {
      const current = await invoke<BongoVersion>("get_current_version");
      console.log("[bongo-nat] Current version:", current.name);
      await loadVersionImages(current);
    } catch (e) {
      console.warn("[bongo-nat] Could not load current version", e);
    }

    isReady = true;

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
    // Listen for background toggle from tray menu
    try {
      unlistenBg = await listen<boolean>("set-background", (event) => {
        showBackground = event.payload;
      });
    } catch (e) {
      console.warn("[bongo-nat] Could not register set-background listener", e);
    }

    // Listen for version changes from tray menu
    try {
      unlistenVersion = await listen<BongoVersion>("version-changed", (event) => {
        console.log("[bongo-nat] Version changed to:", event.payload.name);
        loadVersionImages(event.payload);
      });
    } catch (e) {
      console.warn("[bongo-nat] Could not register version-changed listener", e);
    }

    // Listen for scale changes from tray menu
    try {
      unlistenScale = await listen<number>("scale-changed", (event) => {
        scalePercent = event.payload;
      });
    } catch (e) {
      console.warn("[bongo-nat] Could not register scale-changed listener", e);
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
    if (unlistenBg) unlistenBg();
    if (unlistenVersion) unlistenVersion();
    if (unlistenScale) unlistenScale();
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

{#if isReady}
  <div class="viewport-center">
    <div
      class="nat-container"
      class:no-bg={!showBackground}
      style="--target-size: {300 * (scalePercent / 100)}px;"
      data-tauri-drag-region
    >
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="nat-wrapper"
        data-tauri-drag-region
        bind:this={wrapperEl}
      >
        {#if isBonking}
          {#if bonkCount % 2 === 0}
            <img src={imgLeft} alt="Nat bonking left!" class="nat-image" draggable="false" />
          {:else}
            <img src={imgRight} alt="Nat bonking right!" class="nat-image" draggable="false" />
          {/if}
        {:else}
          <img src={imgIdle} alt="Nat idle" class="nat-image" draggable="false" />
        {/if}
        <div class="bonk-tooltip" class:visible={isHovering}>
          {bonkCount.toLocaleString()}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(*) {
    -webkit-tap-highlight-color: transparent;
    -webkit-touch-callout: none;
    outline: none;
  }

  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent !important;
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
  }

  .viewport-center {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .nat-container {
    --size: min(100vw, 100vh, var(--target-size, 300px));
    width: var(--size);
    height: var(--size);
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 1);
    box-sizing: border-box;
    border: calc(var(--size) * 0.0233) solid #0b0;
    border-radius: 100%;
    padding: calc(var(--size) * 0.1);
    cursor: grab;
  }

  .nat-container:active {
    cursor: grabbing;
  }

  .nat-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }

  .nat-image {
    width: 100%;
    height: 100%;
    object-fit: contain;
    pointer-events: none;
    image-rendering: auto;
    -webkit-user-drag: none;
    user-select: none;
    -webkit-user-select: none;
  }

  .bonk-tooltip {
    position: absolute;
    bottom: 3vh;
    left: 50%;
    transform: translateX(-50%) translateY(-100%);
    background: rgba(255, 255, 255, 0.9);
    color: #000;
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

  .nat-container.no-bg {
    background: transparent;
    border-color: transparent;
  }
</style>
