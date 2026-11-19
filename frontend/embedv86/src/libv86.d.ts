export interface V86Constructor {
  new (config: object): V86;
}
export interface V86 {
  is_running(): boolean;
  destroy(): void;
  run(): Promise<void>;
  stop(): Promise<void>;
  mouse_set_status(on: boolean): void;
  keyboard_set_status(on: boolean): void;
  lock_mouse(): void;
  screen_adapter: ScreenAdapter;
  speaker_adapter: SpeakerAdapter;
  screen_make_screenshot(): HTMLImageElement;
  screen_set_scale(x: number, y: number): void;
  save_state(): Promise<ArrayBuffer>;
  restore_state(sbuf: ArrayBuffer): Promise<void>;
  // eslint-disable-next-line
  add_listener(evt: string, listener: (msg: any) => void): void;
  // eslint-disable-next-line
  remove_listener(evt: string, listener: (msg: any) => void): void;
  get_instruction_counter(): number;
  v86: V86v86;
  bus: Bus;
  emulator_bus: Bus;
}
interface Bus {
  // eslint-disable-next-line
  send(evt: string, val: any): void;
  // eslint-disable-next-line
  register(evt: string, handler: (msg: any) => void): void;
}
interface V86v86 {
  cpu: Cpu;
}
interface Cpu {
  instruction_counter: Uint32Array;
}
interface ScreenAdapter {
  is_graphical: boolean;
  graphic_screen: HTMLCanvasElement;
  cursor_element: HTMLDivElement;
  text_screen: HTMLDivElement;
  text_mode_width: number;
  text_mode_height: number;
  text_mode_data: Int32Array;
  charmap: string[];
  cursor_col: number;
  cursor_row: number;
}
interface SpeakerAdapter {
  mixer: Mixer;
}
interface Mixer {
  node_merger: AudioNode;
  set_volume(vol: number): void;
}
interface V86Config {
  wasm_path: string;
  disable_jit?: boolean;
  bios?: V86Image;
  vga_bios?: V86Image;
  memory_size?: number;
  vga_memory_size?: number;
  screen: ScreenConfig;
  initial_state?: V86Image;
  autostart: boolean;
  fda?: V86Image;
  fdb?: V86Image;
  hda?: V86Image;
  hdb?: V86Image;
  cdrom?: V86Image;
}
interface ScreenConfig {
  container?: HTMLElement | null;
  encoding?: "ascii" | "cp437" | "cp858";
  scaling?: number;
  use_graphical_text?: boolean;
  ansi?: boolean;
}
type V86Image =
  | { buffer: ArrayBuffer | File }
  | {
      url: string;
      size?: number;
      async?: boolean;
      fixed_chunk_size?: number;
      use_parts?: boolean;
    };
