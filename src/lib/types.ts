export type CanonicalButton =
  | "BTN_A"
  | "BTN_B"
  | "BTN_X"
  | "BTN_Y"
  | "BTN_LB"
  | "BTN_RB"
  | "BTN_LT"
  | "BTN_RT"
  | "BTN_BACK"
  | "BTN_START"
  | "BTN_GUIDE"
  | "BTN_LTHUMB"
  | "BTN_RTHUMB"
  | "BTN_DPAD_UP"
  | "BTN_DPAD_DOWN"
  | "BTN_DPAD_LEFT"
  | "BTN_DPAD_RIGHT";

export type CanonicalAxis =
  | "AXIS_LX"
  | "AXIS_LY"
  | "AXIS_RX"
  | "AXIS_RY"
  | "AXIS_LT"
  | "AXIS_RT";

export const BUTTONS: CanonicalButton[] = [
  "BTN_A",
  "BTN_B",
  "BTN_X",
  "BTN_Y",
  "BTN_LB",
  "BTN_RB",
  "BTN_LT",
  "BTN_RT",
  "BTN_BACK",
  "BTN_START",
  "BTN_GUIDE",
  "BTN_LTHUMB",
  "BTN_RTHUMB",
  "BTN_DPAD_UP",
  "BTN_DPAD_DOWN",
  "BTN_DPAD_LEFT",
  "BTN_DPAD_RIGHT",
];

export const AXES: CanonicalAxis[] = [
  "AXIS_LX",
  "AXIS_LY",
  "AXIS_RX",
  "AXIS_RY",
  "AXIS_LT",
  "AXIS_RT",
];

export type LayoutMode = "playstation" | "xbox";
export type ControllerType = "xbox360" | "dualshock4";

export interface AxisMapping {
  physical: CanonicalAxis;
  target: CanonicalAxis;
  invert: boolean;
  deadzone: number;
}

export interface Profile {
  name: string;
  layout: LayoutMode;
  controllerType: ControllerType;
  /** physical canonical -> target canonical */
  buttons: Record<string, string>;
  axes: AxisMapping[];
  turbo: string[];
}

export interface ControllerInfo {
  id: number;
  name: string;
  uuid: string;
  connected: boolean;
  battery: number | null;
  axes: string[];
  buttons: string[];
  isGamepad: boolean;
}

export interface LearnTarget {
  code: CanonicalButton;
  labelPS: string;
  labelXbox: string;
}

export const LEARN_TARGETS: LearnTarget[] = [
  { code: "BTN_A", labelPS: "CROSS", labelXbox: "A" },
  { code: "BTN_B", labelPS: "CIRCLE", labelXbox: "B" },
  { code: "BTN_X", labelPS: "SQUARE", labelXbox: "X" },
  { code: "BTN_Y", labelPS: "TRIANGLE", labelXbox: "Y" },
  { code: "BTN_LB", labelPS: "L1", labelXbox: "LB" },
  { code: "BTN_RB", labelPS: "R1", labelXbox: "RB" },
  { code: "BTN_LT", labelPS: "L2", labelXbox: "LT" },
  { code: "BTN_RT", labelPS: "R2", labelXbox: "RT" },
  { code: "BTN_BACK", labelPS: "SHARE / SELECT", labelXbox: "BACK" },
  { code: "BTN_START", labelPS: "OPTIONS / START", labelXbox: "START" },
  { code: "BTN_LTHUMB", labelPS: "L3 (Stick)", labelXbox: "L3 (Stick)" },
  { code: "BTN_RTHUMB", labelPS: "R3 (Stick)", labelXbox: "R3 (Stick)" },
  { code: "BTN_DPAD_UP", labelPS: "D-Pad Up", labelXbox: "D-Pad Up" },
  { code: "BTN_DPAD_DOWN", labelPS: "D-Pad Down", labelXbox: "D-Pad Down" },
  { code: "BTN_DPAD_LEFT", labelPS: "D-Pad Left", labelXbox: "D-Pad Left" },
  { code: "BTN_DPAD_RIGHT", labelPS: "D-Pad Right", labelXbox: "D-Pad Right" },
];

export interface PadSnapshot {
  id: number;
  axes: Record<string, number>;
  buttons: Record<string, boolean>;
}

export interface MappingStatus {
  emulating: boolean;
  virtualConnected: boolean;
  driverAvailable: boolean;
  error: string | null;
}

const LABELS_XBOX: Record<string, string> = {
  BTN_A: "A",
  BTN_B: "B",
  BTN_X: "X",
  BTN_Y: "Y",
  BTN_LB: "LB",
  BTN_RB: "RB",
  BTN_LT: "LT",
  BTN_RT: "RT",
  BTN_BACK: "BACK",
  BTN_START: "START",
  BTN_GUIDE: "GUIDE",
  BTN_LTHUMB: "L3",
  BTN_RTHUMB: "R3",
  BTN_DPAD_UP: "D-UP",
  BTN_DPAD_DOWN: "D-DOWN",
  BTN_DPAD_LEFT: "D-LEFT",
  BTN_DPAD_RIGHT: "D-RIGHT",
};

const LABELS_PS: Record<string, string> = {
  ...LABELS_XBOX,
  BTN_A: "CROSS",
  BTN_B: "CIRCLE",
  BTN_X: "SQUARE",
  BTN_Y: "TRIANGLE",
  BTN_LB: "L1",
  BTN_RB: "R1",
  BTN_LT: "L2",
  BTN_RT: "R2",
  BTN_BACK: "SHARE",
  BTN_START: "OPTIONS",
  BTN_GUIDE: "PS",
};

export function label(code: string, layout: LayoutMode = "xbox"): string {
  const table = layout === "playstation" ? LABELS_PS : LABELS_XBOX;
  return table[code] ?? code;
}

export const AXIS_LABELS: Record<string, string> = {
  AXIS_LX: "Left X",
  AXIS_LY: "Left Y",
  AXIS_RX: "Right X",
  AXIS_RY: "Right Y",
  AXIS_LT: "Left Trigger",
  AXIS_RT: "Right Trigger",
};

export function defaultProfile(): Profile {
  // Buttons start EMPTY — user binds each physical button explicitly.
  // No auto 1:1 mapping, otherwise every press fires twice.
  return {
    name: "Default",
    layout: "playstation",
    controllerType: "xbox360",
    buttons: {},
    axes: AXES.map((a) => ({
      physical: a,
      target: a,
      invert: a === "AXIS_LY" || a === "AXIS_RY",
      deadzone: 0.1,
    })),
    turbo: [],
  };
}
