import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  defaultProfile,
  LEARN_TARGETS,
  type ControllerInfo,
  type MappingStatus,
  type PadSnapshot,
  type Profile,
} from "./types";

export function normKey(k: string): string {
  const num = k.replace(/\D/g, "");
  return num !== "" ? `Button(${num})` : k;
}

class AppState {
  controllers = $state<ControllerInfo[]>([]);
  snapshots = $state<Record<number, PadSnapshot>>({});
  selectedId = $state<number | null>(null);
  profile = $state<Profile>(defaultProfile());
  profiles = $state<Record<string, Profile>>({});
  activeProfileName = $state<string>("Default");
  status = $state<MappingStatus>({
    emulating: false,
    virtualConnected: false,
    driverAvailable: false,
    error: null,
  });
  ready = $state(false);
  backendError = $state<string | null>(null);
  captureMode = $state<string | null>(null);
  captureBaseline = new Set<string>();
  waitingRelease = false;
  learnIndex = $state<number | null>(null);
  viewMode = $state<"function" | "physical">("function");
  tab = $state<"map" | "monitor" | "profiles" | "settings">("map");
  lastPressed = $state<string | null>(null);

  private unlisten: UnlistenFn[] = [];

  get selected(): ControllerInfo | null {
    if (this.selectedId == null) return null;
    return this.controllers.find((c) => c.id === this.selectedId) ?? null;
  }

  get selectedSnapshot(): PadSnapshot | null {
    if (this.selectedId == null) return null;
    return this.snapshots[this.selectedId] ?? null;
  }

  get currentLearnTarget() {
    if (this.learnIndex === null) return null;
    return LEARN_TARGETS[this.learnIndex] ?? null;
  }

  async init() {
    if (this.ready) return;
    try {
      this.unlisten.push(
        await listen<PadSnapshot[]>("input-update", (e) => {
          const next: Record<number, PadSnapshot> = {};
          for (const snap of e.payload) next[snap.id] = snap;
          this.snapshots = next;

          const sel = this.snapshots[this.selectedId ?? -1];
          if (sel) {
            // Find all physical buttons currently held
            const rawPressed = Object.entries(sel.buttons)
              .filter(([code, isDown]) => isDown && (code.startsWith("Button(") || !isNaN(Number(code))))
              .map(([code]) => normKey(code));

            const uniquePressed = Array.from(new Set(rawPressed));

            if (uniquePressed.length > 0) {
              this.lastPressed = uniquePressed[0];

              if (this.captureMode) {
                if (this.waitingRelease) {
                  // waiting for user to let go of the button
                } else {
                  // Only capture buttons that were NOT pressed when capture began
                  const newPresses = uniquePressed.filter((b) => !this.captureBaseline.has(b));
                  if (newPresses.length > 0) {
                    const chosen = newPresses[0];
                    this.waitingRelease = true;
                    this.applyCapture(chosen);
                  }
                }
              }
            } else {
              // all buttons released
              this.waitingRelease = false;
            }
          }
        }),
      );
      this.unlisten.push(
        await listen<ControllerInfo[]>("controllers-changed", (e) => {
          this.controllers = e.payload;
          if (
            this.selectedId != null &&
            !e.payload.some((c) => c.id === this.selectedId)
          ) {
            this.selectedId = e.payload[0]?.id ?? null;
          }
          if (this.selectedId == null && e.payload.length) {
            this.selectedId = e.payload[0].id;
            void this.loadProfileForSelected();
          }
        }),
      );
      this.unlisten.push(
        await listen<MappingStatus>("status-changed", (e) => {
          this.status = e.payload;
        }),
      );

      await this.refreshControllers();
      await this.refreshProfiles();
      this.status = await invoke<MappingStatus>("get_status");
      this.ready = true;
    } catch (err) {
      this.backendError = String(err);
      this.ready = true;
    }
  }

  dispose() {
    for (const fn of this.unlisten) fn();
    this.unlisten = [];
  }

  async refreshControllers() {
    const list = await invoke<ControllerInfo[]>("list_controllers");
    this.controllers = list;
    if (this.selectedId == null && list.length) {
      this.selectedId = list[0].id;
      await this.loadProfileForSelected();
    }
  }

  async rescan() {
    try {
      const list = await invoke<ControllerInfo[]>("rescan_controllers");
      this.controllers = list;
      if (this.selectedId == null && list.length) {
        this.selectedId = list[0].id;
        await this.loadProfileForSelected();
      }
    } catch (err) {
      this.backendError = String(err);
    }
  }

  async clearDisconnected() {
    try {
      const list = await invoke<ControllerInfo[]>("clear_disconnected");
      this.controllers = list;
      if (!list.some((c) => c.id === this.selectedId)) {
        this.selectedId = list[0]?.id ?? null;
      }
    } catch (err) {
      this.backendError = String(err);
    }
  }

  async refreshProfiles() {
    const map = await invoke<Record<string, Profile>>("list_profiles");
    this.profiles = map;
  }

  async select(id: number) {
    this.selectedId = id;
    await this.loadProfileForSelected();
  }

  async loadProfileForSelected() {
    if (this.selectedId == null) return;
    try {
      const p = await invoke<Profile | null>("get_profile_for_controller", {
        id: this.selectedId,
      });
      this.profile = p ?? this.profiles[this.activeProfileName] ?? defaultProfile();
    } catch {
      this.profile = defaultProfile();
    }
  }

  async startEmulation() {
    if (this.selectedId == null) return;
    try {
      this.status = await invoke<MappingStatus>("start_emulation", {
        id: this.selectedId,
        profile: this.profile,
      });
    } catch (err) {
      this.status = { ...this.status, error: String(err) };
    }
  }

  async stopEmulation() {
    if (this.selectedId == null) return;
    try {
      this.status = await invoke<MappingStatus>("stop_emulation", {
        id: this.selectedId,
      });
    } catch (err) {
      this.status = { ...this.status, error: String(err) };
    }
  }

  async persistProfile() {
    if (this.selectedId == null) return;
    try {
      await invoke("save_profile_for_controller", {
        id: this.selectedId,
        profile: this.profile,
      });
      await this.refreshProfiles();
    } catch (err) {
      this.backendError = String(err);
    }
  }

  /**
   * Set mapping from dropdown in By Button mode
   */
  setMapping(physical: string, target: string) {
    const cleanPhys = normKey(physical);
    const next: Record<string, string> = {};

    // Remove any previous occurrence of this target or this physical button
    for (const [k, v] of Object.entries(this.profile.buttons)) {
      const nk = normKey(k);
      if (nk === cleanPhys || (target && v === target)) {
        continue;
      }
      next[nk] = v;
    }

    if (target) {
      next[cleanPhys] = target;
    }

    this.profile = { ...this.profile, buttons: next };
    void this.persistProfile();
  }

  /**
   * Bind target function to physical button (By Function / Click SVG mode).
   * Guarantees STRICT 1:1 bijection: no duplicate assignments or collisions.
   */
  bindTarget(target: string, physical: string) {
    const cleanPhys = normKey(physical);
    const next: Record<string, string> = {};

    for (const [k, v] of Object.entries(this.profile.buttons)) {
      const nk = normKey(k);
      // If this physical button was used for another function, unbind it
      if (nk === cleanPhys) continue;
      // If this target function was assigned to another physical button, unbind it
      if (v === target) continue;
      next[nk] = v;
    }

    next[cleanPhys] = target;
    this.profile = { ...this.profile, buttons: next };
    void this.persistProfile();
  }

  clearTarget(target: string) {
    const next: Record<string, string> = {};
    for (const [k, v] of Object.entries(this.profile.buttons)) {
      if (v !== target) {
        next[normKey(k)] = v;
      }
    }
    this.profile = { ...this.profile, buttons: next };
    void this.persistProfile();
  }

  clearAllMappings() {
    this.profile = { ...this.profile, buttons: {} };
    void this.persistProfile();
  }

  resetToDefault() {
    this.profile = defaultProfile();
    void this.persistProfile();
  }

  toggleTurbo(physical: string) {
    const clean = normKey(physical);
    const has = this.profile.turbo.some((t) => normKey(t) === clean);
    this.profile = {
      ...this.profile,
      turbo: has
        ? this.profile.turbo.filter((t) => normKey(t) !== clean)
        : [...this.profile.turbo, clean],
    };
    void this.persistProfile();
  }

  updateAxis(physical: string, patch: Partial<{ invert: boolean; deadzone: number; target: string }>) {
    this.profile = {
      ...this.profile,
      axes: this.profile.axes.map((a) =>
        a.physical === physical ? { ...a, ...(patch as any) } : a,
      ),
    };
    void this.persistProfile();
  }

  beginCapture(target: string) {
    this.captureMode = target;
    this.waitingRelease = false;

    // Snapshot currently pressed buttons as baseline so existing held buttons don't fire
    const sel = this.snapshots[this.selectedId ?? -1];
    this.captureBaseline = new Set(
      sel
        ? Object.entries(sel.buttons)
            .filter(([code, isDown]) => isDown && (code.startsWith("Button(") || !isNaN(Number(code))))
            .map(([code]) => normKey(code))
        : [],
    );
  }

  cancelCapture() {
    this.captureMode = null;
    this.learnIndex = null;
    this.captureBaseline.clear();
    this.waitingRelease = false;
  }

  startLearn() {
    this.learnIndex = 0;
    this.beginCapture(LEARN_TARGETS[0].code);
  }

  skipLearn() {
    if (this.learnIndex === null) return;
    this.learnIndex++;
    if (this.learnIndex < LEARN_TARGETS.length) {
      this.beginCapture(LEARN_TARGETS[this.learnIndex].code);
    } else {
      this.stopLearn();
    }
  }

  stopLearn() {
    this.learnIndex = null;
    this.captureMode = null;
    this.captureBaseline.clear();
    this.waitingRelease = false;
  }

  private applyCapture(physical: string) {
    const target = this.captureMode;
    if (!target) return;
    this.bindTarget(target, physical);

    if (this.learnIndex !== null) {
      this.learnIndex++;
      if (this.learnIndex < LEARN_TARGETS.length) {
        this.beginCapture(LEARN_TARGETS[this.learnIndex].code);
      } else {
        this.stopLearn();
      }
    } else {
      this.captureMode = null;
      this.captureBaseline.clear();
    }
  }

  async setProfileMeta(patch: Partial<Profile>) {
    this.profile = { ...this.profile, ...patch };
    await this.persistProfile();
  }
}

export const app = new AppState();
