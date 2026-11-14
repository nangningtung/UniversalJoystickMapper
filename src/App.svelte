<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { app } from "./lib/state.svelte";
  import { normKey } from "./lib/state.svelte";
  import { BUTTONS, AXES, AXIS_LABELS, LEARN_TARGETS, label, type LayoutMode } from "./lib/types";
  import ControllerSVG from "./lib/components/ControllerSVG.svelte";
  import InputMonitor from "./lib/components/InputMonitor.svelte";

  onMount(() => {
    void app.init();
  });
  onDestroy(() => app.dispose());

  const tabs = [
    { id: "map", name: "Mapping" },
    { id: "monitor", name: "Monitor" },
    { id: "profiles", name: "Profiles" },
    { id: "settings", name: "Settings" },
  ] as const;

  function pick(code: string) {
    if (app.captureMode === code) {
      app.cancelCapture();
    } else {
      app.beginCapture(code);
    }
  }

  function shortUuid(u: string) {
    return u.length > 16 ? u.slice(0, 8) + "…" + u.slice(-4) : u;
  }

  const physicalButtons = $derived.by(() => {
    const fromInfo = app.selected?.buttons ?? [];
    const fromProfile = Object.keys(app.profile.buttons);
    const set = new Set([...fromInfo, ...fromProfile]);
    for (let i = 0; i < 12; i++) {
      set.add(`Button(${i})`);
    }
    return Array.from(set).sort((a, b) => {
      const na = parseInt(a.replace(/\D/g, "") || "999");
      const nb = parseInt(b.replace(/\D/g, "") || "999");
      if (na !== nb) return na - nb;
      return a.localeCompare(b);
    });
  });

  function physicalMappedTo(target: string): string | null {
    for (const [phys, t] of Object.entries(app.profile.buttons)) {
      if (t === target) return normKey(phys);
    }
    return null;
  }
</script>

<div class="h-full flex flex-col bg-bg text-fg">
  <!-- top bar -->
  <header class="h-14 shrink-0 flex items-center gap-3 px-5 border-b border-line bg-bg-soft">
    <div class="flex items-center gap-2">
      <div class="w-7 h-7 rounded-lg bg-gradient-to-b from-[#79bbff] to-[#2f81f7] flex items-center justify-center">
        <span class="text-[15px]">🦦</span>
      </div>
      <div class="leading-none">
        <div class="text-[13px] font-bold tracking-[0.14em]">GHOSTPAD</div>
        <div class="text-[9px] text-fg-mute tracking-wider mt-0.5">UNIT-01</div>
      </div>
    </div>

    <div class="flex-1"></div>

    <div class="flex items-center gap-1.5 text-[11px] text-fg-dim">
      <span
        class="relative inline-block w-2 h-2 rounded-full"
        class:bg-ok={app.controllers.length > 0}
        class:bg-fg-mute={app.controllers.length === 0}
      ></span>
      {app.controllers.length} connected
    </div>

    <div
      class="text-[11px] px-2.5 py-1 rounded-md border"
      class:border-ok={app.status.driverAvailable}
      class:text-ok={app.status.driverAvailable}
      class:border-bad={!app.status.driverAvailable}
      class:text-bad={!app.status.driverAvailable}
    >
      {app.status.driverAvailable ? "ViGEmBus ready" : "ViGEmBus missing"}
    </div>
  </header>

  <div class="flex-1 flex min-h-0">
    <!-- sidebar -->
    <aside class="w-[268px] shrink-0 border-r border-line bg-bg-soft flex flex-col min-h-0">
      <div class="px-3 pt-3 pb-2 flex items-center justify-between">
        <span class="text-[10px] uppercase tracking-widest text-fg-mute font-semibold">
          Controllers
        </span>
      </div>

      <div class="px-2 pb-2 flex gap-1.5">
        <button
          class="flex-1 text-[11px] py-1.5 rounded-md bg-panel border border-line hover:border-accent text-fg-dim hover:text-fg flex items-center justify-center gap-1 transition-colors"
          onclick={() => void app.rescan()}
          title="Scan and detect connected joysticks"
        >
          <span>🔄</span> Scan
        </button>
        <button
          class="flex-1 text-[11px] py-1.5 rounded-md bg-panel border border-line hover:border-bad/60 text-fg-dim hover:text-bad flex items-center justify-center gap-1 transition-colors"
          onclick={() => void app.clearDisconnected()}
          title="Clear slots of disconnected joysticks"
        >
          <span>🧹</span> Clear
        </button>
      </div>

      <div class="flex-1 overflow-y-auto px-2 space-y-1.5 pb-3">
        {#if app.controllers.length === 0}
          <div class="text-center text-[12px] text-fg-mute px-4 py-10 leading-relaxed">
            No controller detected.<br />Plug one in and it appears here.
          </div>
        {/if}

        {#each app.controllers as c (c.id)}
          {@const snap = app.snapshots[c.id]}
          <button
            class="w-full text-left rounded-lg border px-3 py-2.5 transition-colors"
            class:border-accent={app.selectedId === c.id}
            class:bg-panel={app.selectedId === c.id}
            class:border-transparent={app.selectedId !== c.id}
            class:hover:bg-panel={app.selectedId !== c.id}
            onclick={() => void app.select(c.id)}
          >
            <div class="flex items-center gap-2">
              <span
                class="relative w-1.5 h-1.5 rounded-full shrink-0"
                class:bg-ok={c.connected}
                class:bg-fg-mute={!c.connected}
                class:live-dot={c.connected}
              ></span>
              <span class="text-[12px] font-medium truncate flex-1">{c.name}</span>
              {#if c.battery !== null}
                <span class="text-[10px] text-fg-mute font-mono">{Math.round(c.battery * 100)}%</span>
              {/if}
            </div>
            <div class="text-[10px] text-fg-mute font-mono mt-1 truncate">
              {shortUuid(c.uuid)} · {c.buttons.length} btn · {c.axes.length} axis
            </div>
          </button>
        {/each}
      </div>

      <div class="border-t border-line p-3 text-[10px] text-fg-mute leading-relaxed">
        <div class="flex justify-between">
          <span>Input rate</span>
          <span class="font-mono">125 Hz</span>
        </div>
        <div class="flex justify-between">
          <span>UI refresh</span>
          <span class="font-mono">60 Hz</span>
        </div>
      </div>
    </aside>

    <!-- main -->
    <main class="flex-1 flex flex-col min-w-0 min-h-0">
      {#if app.backendError}
        <div class="m-4 rounded-lg border border-bad/40 bg-bad/10 px-4 py-3 text-[12px] text-bad">
          Backend error: {app.backendError}
        </div>
      {/if}

      {#if !app.selected}
        <div class="flex-1 flex flex-col items-center justify-center text-center gap-3">
          <div class="text-5xl opacity-30">🎮</div>
          <div class="text-fg-dim text-sm">Connect a controller to begin</div>
          <div class="text-fg-mute text-[11px] max-w-sm leading-relaxed">
            GhostPad reads any DirectInput / XInput device and presents it to games as a
            virtual Xbox 360 controller.
          </div>
        </div>
      {:else}
        <!-- controller header -->
        <div class="shrink-0 px-5 py-3 border-b border-line flex items-center gap-3">
          <div class="min-w-0">
            <div class="text-[15px] font-semibold truncate">{app.selected.name}</div>
            <div class="text-[10px] text-fg-mute font-mono truncate mt-0.5">
              {app.selected.uuid}
            </div>
          </div>

          <div class="flex-1"></div>

          {#if app.status.emulating}
            <div class="flex items-center gap-1.5 text-[11px] text-ok">
              <span class="relative w-1.5 h-1.5 rounded-full bg-ok live-dot"></span>
              Emulating
            </div>
          {/if}

          <button
            class="text-[12px] font-medium px-4 py-2 rounded-lg transition-colors"
            class:bg-ok={!app.status.emulating}
            class:hover:bg-[#34a344]={!app.status.emulating}
            class:text-[#04121f]={!app.status.emulating}
            class:bg-bad={app.status.emulating}
            class:hover:bg-[#e03c34]={app.status.emulating}
            class:text-white={app.status.emulating}
            onclick={() => app.status.emulating ? app.stopEmulation() : app.startEmulation()}
          >
            {app.status.emulating ? "Stop" : "Start Emulation"}
          </button>
        </div>

        {#if app.status.error}
          <div class="mx-5 mt-3 rounded-lg border border-bad/40 bg-bad/10 px-3 py-2 text-[11px] text-bad">
            {app.status.error}
          </div>
        {/if}

        {#if app.learnIndex !== null && app.currentLearnTarget}
          <div class="mx-5 mt-3 rounded-xl border border-ok/50 bg-ok/10 p-4">
            <div class="flex items-center justify-between mb-2">
              <span class="text-xs uppercase font-bold text-ok tracking-wider">
                ⚡ Learn Buttons Wizard ({app.learnIndex + 1} / {LEARN_TARGETS.length})
              </span>
              <div class="flex gap-2">
                <button
                  class="text-[11px] px-2.5 py-1 rounded bg-bg-soft border border-line hover:border-fg-dim"
                  onclick={() => app.skipLearn()}
                >
                  Skip
                </button>
                <button
                  class="text-[11px] px-2.5 py-1 rounded bg-bad/20 text-bad border border-bad/30 hover:bg-bad/30"
                  onclick={() => app.stopLearn()}
                >
                  Cancel
                </button>
              </div>
            </div>
            <div class="text-[15px] font-semibold text-fg">
              Tekan tombol fisik untuk:
              <span class="text-ok font-mono font-bold">
                {app.profile.layout === "playstation" ? app.currentLearnTarget.labelPS : app.currentLearnTarget.labelXbox}
              </span>
            </div>
            <p class="text-[11px] text-fg-dim mt-1">
              Tekan satu tombol pada stick kamu. Aplikasi akan mencatat dan langsung lanjut ke tombol berikutnya.
            </p>
          </div>
        {:else if app.captureMode}
          <div class="mx-5 mt-3 rounded-lg border border-accent/40 bg-accent/10 px-3 py-2 text-[11px] text-accent flex items-center justify-between">
            <span>
              Listening… tekan tombol fisik pada stick untuk
              <b class="font-mono">{label(app.captureMode, app.profile.layout)}</b>
            </span>
            <button class="text-fg-dim hover:text-fg underline" onclick={() => app.cancelCapture()}>
              Batal
            </button>
          </div>
        {/if}

        <!-- tabs -->
        <div class="shrink-0 px-5 pt-3 flex gap-1">
          {#each tabs as t (t.id)}
            <button
              class="text-[12px] px-3.5 py-1.5 rounded-t-md border-b-2 transition-colors"
              class:border-accent={app.tab === t.id}
              class:text-accent={app.tab === t.id}
              class:border-transparent={app.tab !== t.id}
              class:text-fg-dim={app.tab !== t.id}
              class:hover:text-fg={app.tab !== t.id}
              onclick={() => (app.tab = t.id)}
            >
              {t.name}
            </button>
          {/each}
        </div>

        <!-- tab content -->
        <div class="flex-1 min-h-0 overflow-y-auto px-5 py-4 border-t border-line">
          {#if app.tab === "map"}
            <div class="grid grid-cols-1 xl:grid-cols-[minmax(0,1.15fr)_minmax(320px,0.85fr)] gap-5">
              <div class="rounded-xl border border-line bg-panel p-4 flex flex-col">
                <div class="flex items-center justify-between mb-2 pb-2 border-b border-line/60">
                  <span class="text-[11px] font-semibold text-fg-mute uppercase tracking-wider">Visual Controller Preview</span>
                  <div class="flex rounded-md border border-line bg-bg-soft p-0.5 text-[11px]">
                    <button
                      class="px-2.5 py-0.5 rounded font-medium transition-colors"
                      class:bg-accent={app.profile.layout === "playstation"}
                      class:text-[#04121f]={app.profile.layout === "playstation"}
                      class:text-fg-dim={app.profile.layout !== "playstation"}
                      onclick={() => app.setProfileMeta({ layout: "playstation" })}
                    >
                      PlayStation
                    </button>
                    <button
                      class="px-2.5 py-0.5 rounded font-medium transition-colors"
                      class:bg-accent={app.profile.layout === "xbox"}
                      class:text-[#04121f]={app.profile.layout === "xbox"}
                      class:text-fg-dim={app.profile.layout !== "xbox"}
                      onclick={() => app.setProfileMeta({ layout: "xbox" })}
                    >
                      Xbox
                    </button>
                  </div>
                </div>

                <ControllerSVG
                  snapshot={app.selectedSnapshot}
                  profile={app.profile}
                  captureMode={app.captureMode}
                  onpick={pick}
                />
                <p class="text-[10px] text-fg-mute text-center mt-2">
                  Klik tombol apa saja pada gambar stick di atas untuk rebind langsung
                </p>
              </div>

              <div class="space-y-4">
                <section class="rounded-xl border border-line bg-panel p-4">
                  <div class="flex items-center justify-between mb-3">
                    <div class="flex items-center gap-2">
                      <h3 class="text-xs uppercase tracking-wider text-fg-mute font-semibold">
                        Mapping
                      </h3>
                      <div class="flex rounded-md border border-line bg-bg-soft p-0.5 text-[10px]">
                        <button
                          class="px-2 py-0.5 rounded"
                          class:bg-accent={app.viewMode === "function"}
                          class:text-[#04121f]={app.viewMode === "function"}
                          class:text-fg-dim={app.viewMode !== "function"}
                          onclick={() => (app.viewMode = "function")}
                        >
                          By Function
                        </button>
                        <button
                          class="px-2 py-0.5 rounded"
                          class:bg-accent={app.viewMode === "physical"}
                          class:text-[#04121f]={app.viewMode === "physical"}
                          class:text-fg-dim={app.viewMode !== "physical"}
                          onclick={() => (app.viewMode = "physical")}
                        >
                          By Button
                        </button>
                      </div>
                    </div>

                    <div class="flex items-center gap-1.5">
                      <button
                        class="text-[10px] px-2 py-1 rounded border border-line bg-bg-soft hover:border-fg-dim text-fg-dim hover:text-fg transition-colors"
                        onclick={() => app.resetToDefault()}
                        title="Reset all buttons to default PES layout"
                      >
                        ↺ Reset
                      </button>
                      <button
                        class="text-[10px] px-2 py-1 rounded border border-line bg-bg-soft hover:border-bad/60 text-fg-dim hover:text-bad transition-colors"
                        onclick={() => app.clearAllMappings()}
                        title="Clear all button bindings"
                      >
                        ✕ Clear
                      </button>
                      <button
                        class="text-[11px] font-semibold px-2.5 py-1 rounded bg-ok text-[#04121f] hover:bg-[#34a344] transition-colors"
                        onclick={() => app.startLearn()}
                      >
                        ⚡ Learn
                      </button>
                    </div>
                  </div>

                  <div class="flex items-center justify-between text-[11px] px-2 py-1.5 mb-2.5 rounded bg-bg-soft border border-line">
                    <span class="text-fg-dim">Last pressed:</span>
                    <span class="font-mono font-semibold" class:text-accent={!!app.lastPressed} class:text-fg-mute={!app.lastPressed}>
                      {app.lastPressed ?? "none"}
                    </span>
                  </div>

                  {#if app.viewMode === "function"}
                    <div class="space-y-1.5 max-h-[340px] overflow-y-auto pr-1">
                      {#each LEARN_TARGETS as t (t.code)}
                        {@const bound = physicalMappedTo(t.code)}
                        <div class="flex items-center gap-2 px-1 py-1 rounded hover:bg-bg-soft/50">
                          <span class="text-[11px] font-medium text-fg w-[120px] shrink-0 truncate">
                            {app.profile.layout === "playstation" ? t.labelPS : t.labelXbox}
                          </span>
                          <span
                            class="text-[11px] font-mono flex-1 truncate"
                            class:text-ok={!!bound}
                            class:text-fg-mute={!bound}
                          >
                            {bound ?? "Unmapped"}
                          </span>
                          <button
                            class="text-[10px] px-2 py-1 rounded border border-accent/60 text-accent hover:bg-accent/10"
                            class:bg-accent={app.captureMode === t.code}
                            class:text-[#04121f]={app.captureMode === t.code}
                            onclick={() => (app.captureMode === t.code ? app.cancelCapture() : app.beginCapture(t.code))}
                          >
                            {app.captureMode === t.code ? "Press…" : "Change"}
                          </button>
                          {#if bound}
                            <button
                              class="text-[10px] px-1.5 py-1 rounded border border-line text-fg-mute hover:text-bad"
                              onclick={() => app.clearTarget(t.code)}
                              title="Clear binding"
                            >
                              ✕
                            </button>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  {:else}
                    <div class="space-y-1.5 max-h-[340px] overflow-y-auto pr-1">
                      {#each physicalButtons as p (p)}
                        {@const cur = app.profile.buttons[p] ?? ""}
                        <div class="flex items-center gap-2">
                          <span class="text-[11px] font-mono text-fg-dim w-[86px] shrink-0 truncate">
                            {p}
                          </span>
                          <span class="text-fg-mute text-[11px]">→</span>
                          <select
                            class="flex-1 bg-bg-soft border border-line rounded-md px-2 py-1 text-[11px] text-fg outline-none focus:border-accent"
                            value={cur}
                            onchange={(e) => app.setMapping(p, (e.currentTarget as HTMLSelectElement).value)}
                          >
                            <option value="">(None)</option>
                            {#each BUTTONS as target (target)}
                              <option value={target}>{label(target, app.profile.layout)}</option>
                            {/each}
                          </select>
                          <button
                            class="text-[10px] px-2 py-1 rounded border"
                            class:border-accent={app.profile.turbo.includes(p)}
                            class:text-accent={app.profile.turbo.includes(p)}
                            class:border-line={!app.profile.turbo.includes(p)}
                            class:text-fg-mute={!app.profile.turbo.includes(p)}
                            onclick={() => app.toggleTurbo(p)}
                            title="Turbo / rapid fire"
                          >
                            T
                          </button>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </section>

                <section class="rounded-xl border border-line bg-panel p-4">
                  <h3 class="text-xs uppercase tracking-wider text-fg-mute font-semibold mb-3">
                    Axes
                  </h3>
                  <div class="space-y-2">
                    {#each app.profile.axes as ax (ax.physical)}
                      <div class="flex items-center gap-2">
                        <span class="text-[11px] text-fg-dim w-[86px] shrink-0">
                          {AXIS_LABELS[ax.physical]}
                        </span>
                        <input
                          type="range"
                          min="0"
                          max="0.6"
                          step="0.01"
                          value={ax.deadzone}
                          class="flex-1 accent-[#58a6ff]"
                          oninput={(e) =>
                            app.updateAxis(ax.physical, {
                              deadzone: Number((e.currentTarget as HTMLInputElement).value),
                            })}
                        />
                        <span class="text-[10px] font-mono text-fg-mute w-9 text-right">
                          {ax.deadzone.toFixed(2)}
                        </span>
                        <button
                          class="text-[10px] px-2 py-1 rounded border"
                          class:border-accent={ax.invert}
                          class:text-accent={ax.invert}
                          class:border-line={!ax.invert}
                          class:text-fg-mute={!ax.invert}
                          onclick={() => app.updateAxis(ax.physical, { invert: !ax.invert })}
                        >
                          INV
                        </button>
                      </div>
                    {/each}
                  </div>
                </section>
              </div>
            </div>
          {:else if app.tab === "monitor"}
            <InputMonitor snapshot={app.selectedSnapshot} profile={app.profile} />
          {:else if app.tab === "profiles"}
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-5 max-w-4xl">
              <section class="rounded-xl border border-line bg-panel p-5 space-y-4">
                <h3 class="text-xs uppercase tracking-wider text-fg-mute font-semibold">
                  Active Profile
                </h3>

                <label class="block">
                  <span class="text-[11px] text-fg-dim">Name</span>
                  <input
                    class="mt-1 w-full bg-bg-soft border border-line rounded-md px-3 py-2 text-[12px] outline-none focus:border-accent"
                    value={app.profile.name}
                    onchange={(e) =>
                      app.setProfileMeta({ name: (e.currentTarget as HTMLInputElement).value })}
                  />
                </label>

                <div>
                  <span class="text-[11px] text-fg-dim">Label layout</span>
                  <div class="mt-1 flex gap-2">
                    {#each ["playstation", "xbox"] as l (l)}
                      <button
                        class="flex-1 text-[12px] px-3 py-2 rounded-md border capitalize"
                        class:border-accent={app.profile.layout === l}
                        class:text-accent={app.profile.layout === l}
                        class:border-line={app.profile.layout !== l}
                        class:text-fg-dim={app.profile.layout !== l}
                        onclick={() => app.setProfileMeta({ layout: l as LayoutMode })}
                      >
                        {l}
                      </button>
                    {/each}
                  </div>
                </div>

                <div>
                  <span class="text-[11px] text-fg-dim">Virtual controller</span>
                  <div class="mt-1 flex gap-2">
                    {#each [["xbox360", "Xbox 360"], ["dualshock4", "DualShock 4"]] as [v, name] (v)}
                      <button
                        class="flex-1 text-[12px] px-3 py-2 rounded-md border"
                        class:border-accent={app.profile.controllerType === v}
                        class:text-accent={app.profile.controllerType === v}
                        class:border-line={app.profile.controllerType !== v}
                        class:text-fg-dim={app.profile.controllerType !== v}
                        onclick={() => app.setProfileMeta({ controllerType: v as any })}
                      >
                        {name}
                      </button>
                    {/each}
                  </div>
                  <p class="text-[10px] text-fg-mute mt-1.5">
                    DualShock 4 output requires ViGEmBus DS4 support; Xbox 360 is the safe default.
                  </p>
                </div>
              </section>

              <section class="rounded-xl border border-line bg-panel p-5">
                <h3 class="text-xs uppercase tracking-wider text-fg-mute font-semibold mb-3">
                  Saved Profiles
                </h3>
                {#if Object.keys(app.profiles).length === 0}
                  <p class="text-[11px] text-fg-mute">
                    No saved profiles yet. Adjust your mapping — it saves automatically.
                  </p>
                {:else}
                  <div class="space-y-1.5">
                    {#each Object.entries(app.profiles) as [name, p] (name)}
                      <div class="flex items-center justify-between rounded-md border border-line bg-bg-soft px-3 py-2">
                        <div>
                          <div class="text-[12px]">{name}</div>
                          <div class="text-[10px] text-fg-mute font-mono">
                            {p.layout} · {p.controllerType}
                          </div>
                        </div>
                      </div>
                    {/each}
                  </div>
                {/if}
              </section>
            </div>
          {:else}
            <div class="max-w-2xl space-y-4">
              <section class="rounded-xl border border-line bg-panel p-5">
                <h3 class="text-xs uppercase tracking-wider text-fg-mute font-semibold mb-3">
                  Virtual Output
                </h3>
                <div class="text-[12px] space-y-2">
                  <div class="flex justify-between">
                    <span class="text-fg-dim">Driver</span>
                    <span class={app.status.driverAvailable ? "text-ok" : "text-bad"}>
                      {app.status.driverAvailable ? "ViGEmBus detected" : "ViGEmBus not installed"}
                    </span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-fg-dim">Emulation</span>
                    <span class={app.status.emulating ? "text-ok" : "text-fg-mute"}>
                      {app.status.emulating ? "Running" : "Stopped"}
                    </span>
                  </div>
                </div>
                {#if !app.status.driverAvailable}
                  <p class="text-[11px] text-fg-mute mt-3 leading-relaxed">
                    Install ViGEmBus from
                    <span class="text-accent">github.com/nefarius/ViGEmBus</span>
                    to enable virtual controller output.
                  </p>
                {/if}
              </section>

              <section class="rounded-xl border border-line bg-panel p-5">
                <h3 class="text-xs uppercase tracking-wider text-fg-mute font-semibold mb-3">
                  About
                </h3>
                <p class="text-[11px] text-fg-mute leading-relaxed">
                  GhostPad v0.1.0 — universal controller mapper for Windows.<br />
                  Rust + Tauri backend, Svelte 5 frontend.
                </p>
              </section>
            </div>
          {/if}
        </div>
      {/if}
    </main>
  </div>

  {#if !app.ready}
    <div class="absolute inset-0 bg-bg flex items-center justify-center">
      <div class="text-fg-mute text-[12px]">starting engine…</div>
    </div>
  {/if}
</div>
