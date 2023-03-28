import {Replay,ReplayMode,Evt} from './v86replay';
export interface StateInfo {
  name:string;
  thumbnail:string;
}
export interface EmbedV86Config {
  wasm_root:string;
  bios_root:string;
  content_root:string;
  container:HTMLDivElement;
  record_replay:(nom:string)=>void;
  stop_replay:()=>void;
  states_changed:(added:StateInfo[], removed:StateInfo[]) => void;
  replay_checkpoints_changed:(added:StateInfo[], removed:StateInfo[]) => void;
}

export class State {
  name:string;
  state:ArrayBuffer;
  thumbnail:string;
  constructor(name:string, state:ArrayBuffer, thumbnail:string) {
    this.name = name;
    this.thumbnail = thumbnail;
    this.state = state;
  }
}

export class EmbedV86 {
  emulator:V86Starter | null;
  config:EmbedV86Config;
  // TODO wrap in a State class that includes the current replay ID if any
  states:State[];
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
  get_active_replay():Replay {
    nonnull(this.active_replay);
    return this.replays[this.active_replay];
  }
  async save_state() {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      const replay = this.replays[this.active_replay];
      replay.make_checkpoint(this.emulator);
      this.config.replay_checkpoints_changed([replay.checkpoints[replay.checkpoints.length-1]], []);
    } else {
      const screenshot = this.emulator.screen_make_screenshot();
      this.states.push(new State("state"+this.states.length.toString(), await this.emulator.save_state(), screenshot.src));
      this.config.states_changed([this.states[this.states.length-1]], []);
    }
  }
  async record_replay() {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      await this.replays[this.active_replay!].stop(this.emulator);
      this.config.stop_replay();
    }
    this.config.record_replay("replay"+this.replays.length.toString());
    this.active_replay = this.replays.length;
    this.replays.push(await Replay.start_recording(this.emulator));
    this.config.replay_checkpoints_changed(this.replays[this.replays.length-1].checkpoints,[]);
  }
  async stop_replay() {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      await this.replays[this.active_replay].stop(this.emulator);
      this.config.stop_replay();
    }
  }
  async load_state_slot(n:number) {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      const replay = this.replays[this.active_replay];
      const truncated_checkpoints = replay.reset_to_checkpoint(n,replay.mode == ReplayMode.Finished ? ReplayMode.Playback : replay.mode,this.emulator);
      this.config.replay_checkpoints_changed([], truncated_checkpoints);
    } else {
      this.emulator.restore_state(this.states[n].state);
    }
  }
  async play_replay_slot(n:number) {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      await this.replays[this.active_replay].stop(this.emulator);
      this.config.stop_replay();
    }
    this.active_replay = n;
    console.log(this.replays[n].checkpoints);
    await this.replays[n].start_playback(this.emulator);
    this.config.replay_checkpoints_changed(this.replays[n].checkpoints,[]);
  }
  download_file(category:"state" | "save" | "replay", file_name:string):[Blob,string] {
    if(category == "state") {
      const num_str = (file_name.match(/state([0-9]+)$/)?.[1]) ?? "0";
      const save_num = parseInt(num_str,10);
      return [new Blob([this.states[save_num].state]), file_name.toString()+".v86state"];
    } else if(category == "save") {
      throw "Not yet implemented";
    } else if(category == "replay") {
      const num_str = (file_name.match(/replay([0-9]+)$/)?.[1]) ?? "0";
      const replay_num = parseInt(num_str,10);
      return [new Blob([this.replays[replay_num].serialize()]), file_name.toString()+".v86replay"];
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
      const replay = this.replays[this.active_replay];
      let old_cp_count = replay.checkpoints.length;
      replay.tick(this.emulator);
      if(old_cp_count < replay.checkpoints.length) {
        this.config.replay_checkpoints_changed(replay.checkpoints.slice(old_cp_count),[]);
      }
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
function nonnull(obj:number|object|null):asserts obj {
  if(obj == null) {
    throw "Must be non-null";
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

