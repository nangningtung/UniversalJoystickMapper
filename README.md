# GhostPad 🎮👻

Universal controller mapper for Windows. Built with **Tauri v2 (Rust)** and **Svelte 5**.

Presents any DirectInput / XInput gamepad as a virtual Xbox 360 controller to games.

## Quick Start

1. Install [ViGEmBus driver](https://github.com/nefarius/ViGEmBus/releases) (if not already installed).
2. Double-click `start.bat` or run `ghostpad.exe`.
3. Plug in your controller — it appears in the sidebar automatically.
4. Click **Start Emulation** — your controller is now an Xbox 360 pad.

## Features (v0.1.0)

- 🎮 **Universal detection** via `gilrs` (DirectInput, XInput, generic USB/BT pads)
- 📊 **Real-time input visualizer** — 60fps SVG controller diagram + axis/button monitors
- 🎛️ **Click-to-remap** — click any button on the diagram, press physical button to bind
- ⚡ **Turbo / rapid-fire** per button
- 🎯 **Per-axis deadzone & invert** with live preview
- 💾 **Auto-profile per controller GUID** (saves automatically)
- 🔀 **Virtual Xbox 360 output** via ViGEmBus
- 🛡️ **Self-detection filter** — our virtual pad won't loop back as a physical device
- 🪟 **System tray integration** — minimize to tray, restore from tray
- 📦 **Tiny footprint** — **3.7 MB exe**, **1.32 MB installer**, ~25 MB RAM (vs 150 MB Python)

## Build from Source

Requirements:
- Node.js 18+
- Rust 1.77+ (`rustup`)
- Visual Studio Build Tools 2022 (C++ workload)

```bash
# install dependencies
npm install

# run in dev mode with hot reload
npm run tauri dev

# build release exe + installer
npm run tauri build
```

## Architecture

- **Backend**: Rust (`src-tauri/`)
  - `gilrs`: physical gamepad polling (125 Hz thread)
  - `vigem.rs`: dynamic loader for `ViGEmClient.dll`
  - `mapping.rs`: transforms input state to virtual Xbox report
  - `profile.rs`: JSON profile persistence per controller UUID
- **Frontend**: Svelte 5 + Tailwind CSS v4 (`src/`)
  - `ControllerSVG.svelte`: responsive vector controller diagram
  - `InputMonitor.svelte`: axis bars and button matrix
  - `state.svelte.ts`: reactive state via Svelte 5 runes (`$state`)

## License

MIT / Open Source — feel free to modify and share.
