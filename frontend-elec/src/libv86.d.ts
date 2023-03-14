declare class V86Starter {
  constructor(config:object);
  is_running(): bool;
  screen_adapter:ScreenAdapter;
  screen_make_screenshot():Image;
  async save_state(): ArrayBuffer;
  async restore_state(ArrayBuffer): void;
}
declare class ScreenAdapter {
  is_graphical:boolean;
  graphic_screen:HTMLCanvasElement;
  cursor_element:HTMLDivElement;
  text_screen:HTMLDivElement;
  text_mode_width:number;
  text_mode_height:number;
  text_mode_data:Int32Array;
  charmap:string[];
  cursor_col:number;
  cursor_row:number;
}
interface V86StarterConfig {
  wasm_path:string,
  bios?:V86Image,
  vga_bios?:V86Image,
  memory_size?:int,
  vga_memory_size?:int,
  screen_container:HTMLDivElement,
  initial_state?:{url:string},
  autostart:boolean,
  fda?:V86Image,
  fdb?:V86Image,
  hda?:V86Image,
  hdb?:V86Image,
  cdrom?:V86Image,
}
type V86Image = {buffer:ArrayBuffer|File} | {url:string} | {url:string, size:number, async:boolean, fixed_chunk_size?: number, use_parts?: boolean};
