<script lang="ts">
  import type { PadSnapshot, Profile } from "../types";
  import { label } from "../types";

  interface Props {
    snapshot: PadSnapshot | null;
    profile: Profile;
    captureMode?: string | null;
    onpick?: (code: string) => void;
  }

  let { snapshot, profile, captureMode = null, onpick }: Props = $props();

  const isPS = $derived(profile.layout === "playstation");

  const canonPhys = (phys: string) => (/^\d+$/.test(phys) ? `Button(${phys})` : phys);
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

  const axis = (code: string) => snapshot?.axes?.[code] ?? 0;

  const lx = $derived(axis("AXIS_LX"));
  const ly = $derived(axis("AXIS_LY"));
  const rx = $derived(axis("AXIS_RX"));
  const ry = $derived(axis("AXIS_RY"));

  const ltValue = $derived(
    isVirtualActive("BTN_LT")
      ? 1.0
      : Math.max(0, Math.min(1, axis("AXIS_LT"))),
  );
  const rtValue = $derived(
    isVirtualActive("BTN_RT")
      ? 1.0
      : Math.max(0, Math.min(1, axis("AXIS_RT"))),
  );

  const isCapturing = (code: string) => captureMode === code;

  function click(code: string) {
    onpick?.(code);
  }

  const faceButtonsPS = [
    { code: "BTN_Y", relX: 0, relY: -38, c: "#2ecc71", t: "△" },
    { code: "BTN_B", relX: 38, relY: 0, c: "#e74c3c", t: "○" },
    { code: "BTN_A", relX: 0, relY: 38, c: "#3498db", t: "✕" },
    { code: "BTN_X", relX: -38, relY: 0, c: "#e91e63", t: "□" },
  ];

  const faceButtonsXbox = [
    { code: "BTN_Y", relX: 0, relY: -38, c: "#f1c40f", t: "Y" },
    { code: "BTN_B", relX: 38, relY: 0, c: "#e74c3c", t: "B" },
    { code: "BTN_A", relX: 0, relY: 38, c: "#2ecc71", t: "A" },
    { code: "BTN_X", relX: -38, relY: 0, c: "#3498db", t: "X" },
  ];

  const faceButtons = $derived(isPS ? faceButtonsPS : faceButtonsXbox);

  // Layout slots
  // PlayStation: D-pad top-left, Left Stick bottom-left
  // Xbox: Left Stick top-left, D-pad bottom-left
  const dpadPos = $derived(isPS ? { x: 180, y: 156 } : { x: 180, y: 258 });
  const leftStickPos = $derived(isPS ? { x: 180, y: 258 } : { x: 180, y: 156 });
  const facePos = { x: 440, y: 156 };
  const rightStickPos = { x: 440, y: 258 };
</script>

<svg viewBox="0 0 620 390" class="w-full h-auto select-none" role="img">
  <defs>
    <linearGradient id="bodyGrad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#1e2430" />
      <stop offset="100%" stop-color="#11151d" />
    </linearGradient>
    <linearGradient id="neonBlue" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#79bbff" />
      <stop offset="100%" stop-color="#1f6feb" />
    </linearGradient>
    <filter id="glowNeon" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur stdDeviation="5" result="b" />
      <feMerge>
        <feMergeNode in="b" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
  </defs>

  <!-- L2 Trigger (analog bar) -->
  <g class="cursor-pointer" onclick={() => click("BTN_LT")} role="button" tabindex="0">
    <rect x="110" y="24" width="120" height="26" rx="13" fill="#151b24" stroke={isCapturing("BTN_LT") ? "#58a6ff" : "#2d3748"} stroke-width="2" />
    {#if ltValue > 0.01}
      <rect x="110" y="24" width={120 * ltValue} height="26" rx="13" fill="#3a7ebf" opacity="0.9" />
    {/if}
    <text x="170" y="42" text-anchor="middle" font-size="12" font-weight="700" fill={ltValue > 0.35 ? "#04121f" : "#a0aec0"}>
      {isPS ? "L2" : "LT"}
    </text>
  </g>

  <!-- R2 Trigger (analog bar) -->
  <g class="cursor-pointer" onclick={() => click("BTN_RT")} role="button" tabindex="0">
    <rect x="390" y="24" width="120" height="26" rx="13" fill="#151b24" stroke={isCapturing("BTN_RT") ? "#58a6ff" : "#2d3748"} stroke-width="2" />
    {#if rtValue > 0.01}
      <rect x="390" y="24" width={120 * rtValue} height="26" rx="13" fill="#3a7ebf" opacity="0.9" />
    {/if}
    <text x="450" y="42" text-anchor="middle" font-size="12" font-weight="700" fill={rtValue > 0.35 ? "#04121f" : "#a0aec0"}>
      {isPS ? "R2" : "RT"}
    </text>
  </g>

  <!-- L1 Shoulder Bumper -->
  <g class="cursor-pointer" onclick={() => click("BTN_LB")} role="button" tabindex="0">
    <rect x="110" y="58" width="120" height="28" rx="14"
      fill={isVirtualActive("BTN_LB") ? "#4caf50" : "#1a212b"}
      stroke={isCapturing("BTN_LB") ? "#58a6ff" : "#2d3748"} stroke-width="2"
      filter={isVirtualActive("BTN_LB") ? "url(#glowNeon)" : undefined} />
    <text x="170" y="77" text-anchor="middle" font-size="12" font-weight="700"
      fill={isVirtualActive("BTN_LB") ? "#04121f" : "#cbd5e1"}>
      {isPS ? "L1" : "LB"}
    </text>
  </g>

  <!-- R1 Shoulder Bumper -->
  <g class="cursor-pointer" onclick={() => click("BTN_RB")} role="button" tabindex="0">
    <rect x="390" y="58" width="120" height="28" rx="14"
      fill={isVirtualActive("BTN_RB") ? "#4caf50" : "#1a212b"}
      stroke={isCapturing("BTN_RB") ? "#58a6ff" : "#2d3748"} stroke-width="2"
      filter={isVirtualActive("BTN_RB") ? "url(#glowNeon)" : undefined} />
    <text x="450" y="77" text-anchor="middle" font-size="12" font-weight="700"
      fill={isVirtualActive("BTN_RB") ? "#04121f" : "#cbd5e1"}>
      {isPS ? "R1" : "RB"}
    </text>
  </g>

  <!-- Controller Body -->
  <path
    d="M 310 92
       C 260 92 230 102 200 110
       C 150 122 105 116 80 144
       C 52 176 58 234 76 274
       C 94 314 135 344 178 344
       C 218 344 240 318 270 304
       C 290 294 330 294 350 304
       C 380 318 402 344 442 344
       C 485 344 526 314 544 274
       C 562 234 568 176 540 144
       C 515 116 470 122 420 110
       C 390 102 360 92 310 92 Z"
    fill="url(#bodyGrad)" stroke="#2d3748" stroke-width="2.5" />

  <!-- Touchpad (PlayStation) / Xbox Center -->
  {#if isPS}
    <rect x="250" y="105" width="120" height="54" rx="8" fill="#151b24" stroke="#2d3748" stroke-width="1.5" />
    <text x="310" y="136" text-anchor="middle" font-size="10" font-weight="600" fill="#64748b" letter-spacing="2">TOUCHPAD</text>
  {:else}
    <g class="cursor-pointer" onclick={() => click("BTN_GUIDE")} role="button" tabindex="0">
      <circle cx="310" cy="132" r="18" fill={isVirtualActive("BTN_GUIDE") ? "#4caf50" : "#1a212b"} stroke="#2d3748" stroke-width="2" />
      <text x="310" y="137" text-anchor="middle" font-size="14" font-weight="800" fill={isVirtualActive("BTN_GUIDE") ? "#04121f" : "#64748b"}>✕</text>
    </g>
  {/if}

  <!-- Select / Share button -->
  <g class="cursor-pointer" onclick={() => click("BTN_BACK")} role="button" tabindex="0">
    <rect x="240" y="180" width="52" height="24" rx="12"
      fill={isVirtualActive("BTN_BACK") ? "#4caf50" : "#1a212b"}
      stroke={isCapturing("BTN_BACK") ? "#58a6ff" : "#2d3748"} stroke-width="1.5"
      filter={isVirtualActive("BTN_BACK") ? "url(#glowNeon)" : undefined} />
    <text x="266" y="196" text-anchor="middle" font-size="9" font-weight="700"
      fill={isVirtualActive("BTN_BACK") ? "#04121f" : "#94a3b8"}>
      {isPS ? "SHARE" : "BACK"}
    </text>
  </g>

  <!-- Start / Options button -->
  <g class="cursor-pointer" onclick={() => click("BTN_START")} role="button" tabindex="0">
    <rect x="328" y="180" width="52" height="24" rx="12"
      fill={isVirtualActive("BTN_START") ? "#4caf50" : "#1a212b"}
      stroke={isCapturing("BTN_START") ? "#58a6ff" : "#2d3748"} stroke-width="1.5"
      filter={isVirtualActive("BTN_START") ? "url(#glowNeon)" : undefined} />
    <text x="354" y="196" text-anchor="middle" font-size="9" font-weight="700"
      fill={isVirtualActive("BTN_START") ? "#04121f" : "#94a3b8"}>
      {isPS ? "OPTIONS" : "START"}
    </text>
  </g>

  <!-- PS / Guide Home Button (PlayStation center) -->
  {#if isPS}
    <g class="cursor-pointer" onclick={() => click("BTN_GUIDE")} role="button" tabindex="0">
      <circle cx="310" cy="226" r="14"
        fill={isVirtualActive("BTN_GUIDE") ? "#3498db" : "#1a212b"}
        stroke={isCapturing("BTN_GUIDE") ? "#58a6ff" : "#2d3748"} stroke-width="1.5" />
      <text x="310" y="230.5" text-anchor="middle" font-size="10" font-weight="800"
        fill={isVirtualActive("BTN_GUIDE") ? "#04121f" : "#64748b"}>PS</text>
    </g>
  {/if}

  <!-- D-PAD Component -->
  <g transform="translate({dpadPos.x} {dpadPos.y})">
    {#each [
      { code: "BTN_DPAD_UP", x: -12, y: -40, w: 24, h: 30, t: "▲" },
      { code: "BTN_DPAD_DOWN", x: -12, y: 10, w: 24, h: 30, t: "▼" },
      { code: "BTN_DPAD_LEFT", x: -40, y: -12, w: 30, h: 24, t: "◀" },
      { code: "BTN_DPAD_RIGHT", x: 10, y: -12, w: 30, h: 24, t: "▶" },
    ] as d (d.code)}
      <g class="cursor-pointer" onclick={() => click(d.code)} role="button" tabindex="0">
        <rect x={d.x} y={d.y} width={d.w} height={d.h} rx="5"
          fill={isVirtualActive(d.code) ? "#4caf50" : "#161c25"}
          stroke={isCapturing(d.code) ? "#58a6ff" : "#2d3748"} stroke-width="1.5"
          filter={isVirtualActive(d.code) ? "url(#glowNeon)" : undefined} />
        <text x={d.x + d.w / 2} y={d.y + d.h / 2 + 4.5} text-anchor="middle" font-size="11" font-weight="bold"
          fill={isVirtualActive(d.code) ? "#04121f" : "#94a3b8"}>{d.t}</text>
      </g>
    {/each}
    <rect x="-12" y="-12" width="24" height="24" fill="#12171f" stroke="#232c3a" stroke-width="1" rx="4" />
  </g>

  <!-- Left Analog Stick -->
  <g class="cursor-pointer" onclick={() => click("BTN_LTHUMB")} role="button" tabindex="0">
    <circle cx={leftStickPos.x} cy={leftStickPos.y} r="42" fill="#0d1117" stroke="#2d3748" stroke-width="2.5" />
    <circle cx={leftStickPos.x} cy={leftStickPos.y} r="28" fill="none" stroke="#1c2432" stroke-width="1" stroke-dasharray="2 3" />
    <circle
      cx={leftStickPos.x + lx * 24}
      cy={leftStickPos.y - ly * 24}
      r={isVirtualActive("BTN_LTHUMB") ? 19 : 17}
      fill={isVirtualActive("BTN_LTHUMB") ? "#4caf50" : "#252e3c"}
      stroke={isCapturing("BTN_LTHUMB") ? "#58a6ff" : "#4b596d"}
      stroke-width="2.5"
      filter={isVirtualActive("BTN_LTHUMB") ? "url(#glowNeon)" : undefined}
    />
    <text x={leftStickPos.x} y={leftStickPos.y + 54} text-anchor="middle" font-size="10" font-weight="600" fill="#64748b">
      L3
    </text>
  </g>

  <!-- Right Analog Stick -->
  <g class="cursor-pointer" onclick={() => click("BTN_RTHUMB")} role="button" tabindex="0">
    <circle cx={rightStickPos.x} cy={rightStickPos.y} r="42" fill="#0d1117" stroke="#2d3748" stroke-width="2.5" />
    <circle cx={rightStickPos.x} cy={rightStickPos.y} r="28" fill="none" stroke="#1c2432" stroke-width="1" stroke-dasharray="2 3" />
    <circle
      cx={rightStickPos.x + rx * 24}
      cy={rightStickPos.y - ry * 24}
      r={isVirtualActive("BTN_RTHUMB") ? 19 : 17}
      fill={isVirtualActive("BTN_RTHUMB") ? "#4caf50" : "#252e3c"}
      stroke={isCapturing("BTN_RTHUMB") ? "#58a6ff" : "#4b596d"}
      stroke-width="2.5"
      filter={isVirtualActive("BTN_RTHUMB") ? "url(#glowNeon)" : undefined}
    />
    <text x={rightStickPos.x} y={rightStickPos.y + 54} text-anchor="middle" font-size="10" font-weight="600" fill="#64748b">
      R3
    </text>
  </g>

  <!-- Face Action Buttons (Triangle, Circle, Cross, Square / Y, B, A, X) -->
  {#each faceButtons as b (b.code)}
    {@const bx = facePos.x + b.relX}
    {@const by = facePos.y + b.relY}
    {@const active = isVirtualActive(b.code)}
    <g class="cursor-pointer" onclick={() => click(b.code)} role="button" tabindex="0">
      <circle cx={bx} cy={by} r="18"
        fill={active ? b.c : "#1a212b"}
        stroke={isCapturing(b.code) ? "#58a6ff" : active ? b.c : "#2d3748"} stroke-width="2"
        filter={active ? "url(#glowNeon)" : undefined} />
      <text x={bx} y={by + 5.5} text-anchor="middle" font-size="15" font-weight="800"
        fill={active ? "#04121f" : "#cbd5e1"}>
        {b.t}
      </text>
    </g>
  {/each}
</svg>
