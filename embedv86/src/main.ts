import {Replay,Evt} from './v86replay';
export interface EmbedV86Config {
  wasm_root:string;
  bios_root:string;
  content_root:string;
  container:HTMLDivElement;
}

export class EmbedV86 {
  emulator:V86Starter | null;
  config:EmbedV86Config;
  // TODO wrap in a State class that includes the current replay ID if any
  states:ArrayBuffer[];
  replays:Replay[];
  active_replay:number|null;
  constructor(config:EmbedV86Config) {
    this.active_replay = null;
    this.config = config;
    this.states = [];
    this.replays = [];
    this.emulator = null;
  }
  clear() {
    this.states = [];
    this.replays = [];
    this.active_replay = null;
    if(this.emulator) {
      this.emulator.destroy();
      this.emulator = null;
    }
  }
  async save_state(callback:(nom:string,screen:string)=>void) {
    nonnull(this.emulator);
    const screenshot = this.emulator.screen_make_screenshot();
    callback("state"+this.states.length.toString(), screenshot.src);
    this.states.push(await this.emulator.save_state());
  }
  async record_replay(callback:(nom:string)=>void) {
    nonnull(this.emulator);
    //const screenshot = emulator.screen_make_screenshot();
    if(this.active_replay != null) {
      await this.replays[this.active_replay!].stop(this.emulator);
    }
    callback("replay"+this.replays.length.toString());
    this.active_replay = this.replays.length;
    this.replays.push(await Replay.start_recording(this.emulator));
  }
  async stop_replay() {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      await this.replays[this.active_replay].stop(this.emulator);
    }
  }
  async load_state_slot(n:number) {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      console.log("loading states during replay recording/playback is not yet supported for v86");
      await this.replays[this.active_replay].stop(this.emulator);
    }
    this.emulator.restore_state(this.states[n]);
  }
  async play_replay_slot(n:number) {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      await this.replays[this.active_replay].stop(this.emulator);
    }
    this.active_replay = n;
    await this.replays[n].start_playback(this.emulator);
  }
  download_file(category:"state" | "save" | "replay", file_name:string):[Blob,string] {
    if(category == "state") {
      const num_str = (file_name.match(/state([0-9]+)$/)?.[1]) ?? "0";
      const save_num = parseInt(num_str,10);
      return [new Blob([this.states[save_num]]), file_name.toString()+".v86state"];
    } else if(category == "save") {
      throw "Not yet implemented";
    } else if(category == "replay") {
      // const num_str = (file_name.match(/replay([0-9]+)$/)?.[1]) ?? "0";
      // const replay_num = parseInt(num_str,10);
      // saveAs(new Blob([this.replays[replay_num]]), file_name.toString()+".v86replay");
      throw "Not yet implemented";
    } else {
      throw "Invalid save category";
    }
  }
  replay_log(evt:Evt, val:any) {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      this.replays[this.active_replay].log_evt(this.emulator, evt, val);
    }
  }
  replay_tick() {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      this.replays[this.active_replay].tick(this.emulator);
    }
  }
  async run(content:string, entryState:boolean, movie:boolean) {
    const content_folder = this.config.content_root;
    const config:V86StarterConfig = {
      wasm_path: this.config.wasm_root+"/v86.wasm",
      screen_container:this.config.container,
      autostart: true
    };
    if(entryState) {
      config["initial_state"] = {url:content_folder+"/entry_state"}
    }
    if(movie) {
      // do nothing for now
    }
    const content_resp = await fetch(content_folder+"/"+content);
    if(!content_resp.ok) { alert("Failed to load content"); return; }
    const content_json = await content_resp.json();
    setup_image("bios", content_json, config, this.config.bios_root);
    setup_image("vga_bios", content_json, config, this.config.bios_root);
    setup_image("fda", content_json, config, content_folder);
    setup_image("fdb", content_json, config, content_folder);
    setup_image("hda", content_json, config, content_folder);
    setup_image("hdb", content_json, config, content_folder);
    setup_image("cdrom", content_json, config, content_folder);

    this.emulator = new V86Starter(config);
    this.emulator.emulator_bus.register("keyboard-code", (k:number) => this.replay_log(Evt.KeyCode,k));
    this.emulator.emulator_bus.register("mouse-click", (v:[boolean,boolean,boolean]) => this.replay_log(Evt.MouseClick,v));
    this.emulator.emulator_bus.register("mouse-delta", (delta:[number,number]) => this.replay_log(Evt.MouseDelta,delta));
    this.emulator.emulator_bus.register("mouse-absolute", (pos:[number,number,number,number]) => this.replay_log(Evt.MouseAbsolute,pos));
    this.emulator.emulator_bus.register("mouse-wheel", (delta:[number,number]) => this.replay_log(Evt.MouseWheel, delta));
    this.emulator.bus.register("emulator-ticked", (_k:any) => this.replay_tick());
  }
}
function nonnull(emu:V86Starter|null):asserts emu {
  if(emu == null) {
    throw "Emulator must be non-null";
  }
}
function setup_image(img:"bios"|"vga_bios"|"fda"|"fdb"|"hda"|"hdb"|"cdrom", content_json:any, config:V86StarterConfig, content_folder:string) {
  if(img in content_json) {
    if("url" in content_json[img]) {
      content_json[img]["url"] = content_folder+"/"+content_json[img]["url"];
    }
    if(!("async" in content_json[img])) {
      content_json[img]["async"] = false;
    }
    config[img] = content_json[img];
  }
}

