<script lang="ts">
  import type { PadSnapshot, Profile } from "../types";
  import { AXIS_LABELS, BUTTONS, label } from "../types";

  interface Props {
    snapshot: PadSnapshot | null;
    profile: Profile;
  }

  let { snapshot, profile }: Props = $props();

  const axisValue = (code: string) => snapshot?.axes?.[code] ?? 0;

  // Mirror of the Rust mapping logic: one canonical physical key,
  // first binding to claim a target wins, aliases never double-fire.
  const canonPhys = (phys: string) =>
    /^\d+$/.test(phys) ? `Button(${phys})` : phys;

  const isPhysPressed = (phys: string) => {
    if (!snapshot?.buttons) return false;
    if (snapshot.buttons[phys]) return true;
    const num = phys.replace(/\D/g, "");
    if (num && snapshot.buttons[`Button(${num})`]) return true;
    if (num && snapshot.buttons[num]) return true;
    return false;
  };

  const isVirtualActive = (targetCode: string) => {
    if (!profile?.buttons) return false;

    const seen = new Set<string>();
    const claimed = new Set<string>();

    for (const [phys, target] of Object.entries(profile.buttons)) {
      const canon = canonPhys(phys);
      if (seen.has(canon)) continue;
      seen.add(canon);

      if (isPhysPressed(phys)) {
        if (claimed.has(target)) continue;
        claimed.add(target);
        if (target === targetCode) return true;
      }
    }

    if (targetCode.startsWith("BTN_DPAD_")) {
      const isOverridden = Object.values(profile.buttons).includes(targetCode);
      if (!isOverridden) {
        if (targetCode === "BTN_DPAD_UP" && snapshot?.buttons?.["RAW_POV_UP"]) return true;
        if (targetCode === "BTN_DPAD_DOWN" && snapshot?.buttons?.["RAW_POV_DOWN"]) return true;
        if (targetCode === "BTN_DPAD_LEFT" && snapshot?.buttons?.["RAW_POV_LEFT"]) return true;
        if (targetCode === "BTN_DPAD_RIGHT" && snapshot?.buttons?.["RAW_POV_RIGHT"]) return true;
      }
    }

    return false;
  };

  const isPhysActive = (code: string) => isPhysPressed(code);

  const stickAxes = ["AXIS_LX", "AXIS_LY", "AXIS_RX", "AXIS_RY"];
  const triggerAxes = ["AXIS_LT", "AXIS_RT"];
  const physicalButtonList = Array.from({ length: 12 }, (_, i) => `Button(${i})`);
</script>

<div class="grid grid-cols-1 xl:grid-cols-2 gap-5">
  <section class="rounded-xl border border-line bg-panel p-5">
    <h3 class="text-xs uppercase tracking-wider text-fg-mute font-semibold mb-4">Analog Sticks</h3>
    <div class="space-y-3">
      {#each stickAxes as a (a)}
        <div>
          <div class="flex justify-between text-[11px] mb-1">
            <span class="text-fg-dim">{AXIS_LABELS[a]}</span>
            <span class="font-mono text-fg-mute">{axisValue(a).toFixed(3)}</span>
          </div>
          <div class="h-2 rounded-full bg-bg-soft overflow-hidden relative">
            <div class="absolute top-0 bottom-0 left-1/2 w-px bg-line"></div>
            <div
              class="absolute top-0 bottom-0 bg-accent transition-[width,left] duration-75"
              style="left: {Math.min(50, (axisValue(a) + 1) * 50)}%; width: {Math.abs(axisValue(a)) * 50}%"
            ></div>
          </div>
        </div>
      {/each}
    </div>
  </section>

  <section class="rounded-xl border border-line bg-panel p-5">
    <h3 class="text-xs uppercase tracking-wider text-fg-mute font-semibold mb-4">Triggers</h3>
    <div class="space-y-4">
      {#each triggerAxes as a (a)}
        {@const v = isVirtualActive(a === "AXIS_LT" ? "BTN_LT" : "BTN_RT") ? 1.0 : Math.max(0, Math.min(1, axisValue(a)))}
        <div>
          <div class="flex justify-between text-[11px] mb-1">
            <span class="text-fg-dim">{AXIS_LABELS[a]}</span>
            <span class="font-mono text-fg-mute">{v.toFixed(3)}</span>
          </div>
          <div class="h-3 rounded-full bg-bg-soft overflow-hidden">
            <div class="h-full bg-accent transition-[width] duration-75" style="width: {v * 100}%"></div>
          </div>
        </div>
      {/each}
    </div>
  </section>

  <section class="rounded-xl border border-line bg-panel p-5 xl:col-span-2">
    <h3 class="text-xs uppercase tracking-wider text-fg-mute font-semibold mb-4">Physical Buttons (Raw Stick Switches)</h3>
    <div class="grid grid-cols-3 sm:grid-cols-4 lg:grid-cols-6 gap-2 mb-6">
      {#each physicalButtonList as pb (pb)}
        <div
          class="rounded-lg px-3 py-2 text-center text-[11px] font-mono font-medium border transition-colors"
          class:bg-ok={isPhysActive(pb)}
          class:text-[#04121f]={isPhysActive(pb)}
          class:border-ok={isPhysActive(pb)}
          class:border-line={!isPhysActive(pb)}
          class:bg-bg-soft={!isPhysActive(pb)}
          class:text-fg-dim={!isPhysActive(pb)}
        >
          {pb}
        </div>
      {/each}
    </div>

    <h3 class="text-xs uppercase tracking-wider text-fg-mute font-semibold mb-4">Virtual Controller Output (What Games Receive)</h3>
    <div class="grid grid-cols-3 sm:grid-cols-4 lg:grid-cols-6 gap-2">
      {#each BUTTONS as b (b)}
        <div
          class="rounded-lg px-3 py-2 text-center text-[11px] font-medium border transition-colors"
          class:bg-accent={isVirtualActive(b)}
          class:text-[#04121f]={isVirtualActive(b)}
          class:border-accent={isVirtualActive(b)}
          class:border-line={!isVirtualActive(b)}
          class:bg-bg-soft={!isVirtualActive(b)}
          class:text-fg-dim={!isVirtualActive(b)}
        >
          {label(b, profile.layout)}
        </div>
      {/each}
    </div>
  </section>
</div>
