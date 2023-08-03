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
  register_replay:(nom:string)=>void;
  stop_replay:()=>void;
  states_changed:(added:StateInfo[], removed:StateInfo[]) => void;
  replay_checkpoints_changed:(added:StateInfo[], removed:StateInfo[]) => void;
}
export interface ConfigSettings {
  bios?:V86Image;
  vga_bios?:V86Image;
  fda?:V86Image;
  fdb?:V86Image;
  hda?:V86Image;
  hdb?:V86Image;
  cdrom?:V86Image;
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
    this.config.register_replay("replay"+this.replays.length.toString());
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
  async download_file(category:"state" | "save" | "replay", file_name:string):Promise<[Blob,string]> {
    if(category == "state") {
      const num_str = (file_name.match(/state([0-9]+)$/)?.[1]) ?? "0";
      const save_num = parseInt(num_str,10);
      return [new Blob([this.states[save_num].state]), file_name.toString()+".v86state"];
    } else if(category == "save") {
      throw "Not yet implemented";
    } else if(category == "replay") {
      const num_str = (file_name.match(/replay([0-9]+)$/)?.[1]) ?? "0";
      const replay_num = parseInt(num_str,10);
      const rep = this.replays[replay_num];
      const ser_rep = await rep.serialize();
      //TODO remove me, just for testing
      const unser_rep = await Replay.deserialize(ser_rep);
      if(unser_rep.events.length != rep.events.length || unser_rep.checkpoints.length != rep.checkpoints.length) {
        throw "ser roundtrip error";
      }
      return [new Blob([ser_rep]), file_name.toString()+".v86replay"];
    } else {
      throw "Invalid save category";
    }
  }
  replay_log(evt:Evt, val:number|object) {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      this.replays[this.active_replay].log_evt(this.emulator, evt, val);
    }
  }
  replay_tick() {
    nonnull(this.emulator);
    if(this.active_replay != null) {
      const replay = this.replays[this.active_replay];
      const old_cp_count = replay.checkpoints.length;
      replay.tick(this.emulator);
      if(old_cp_count < replay.checkpoints.length) {
        this.config.replay_checkpoints_changed(replay.checkpoints.slice(old_cp_count),[]);
      }
    }
  }
  async run(content:ConfigSettings|string, entryState:string|null, movie:string|null) {
    this.clear();
    const content_folder = this.config.content_root;
    const config:V86StarterConfig = {
      wasm_path: this.config.wasm_root+"/v86.wasm",
      screen_container:this.config.container,
      autostart: true
    };
    if(entryState && movie) {
      throw "Can't specify both entry state and movie";
    }
    // TODO: avoid use of /, get explicit paths or a path joining function as arguments or config props
    if(entryState) {
      config["initial_state"] = {url:content_folder+"/"+entryState}
    }
    if(movie) {
      // do nothing for now
      const replay_resp = await fetch(content_folder+"/"+movie);
      if(!replay_resp.ok) { alert("Failed to load replay movie"); return; }
      const replay_data = await replay_resp.arrayBuffer();
      const replay = await Replay.deserialize(replay_data);
      console.log(replay.id,replay.events.length,replay.checkpoints.length);
      this.config.register_replay("replay"+this.replays.length.toString());
      this.replays.push(replay);
    }
    let content_json;
    if (typeof content == "string") {
      const content_resp = await fetch(content_folder+"/"+content);
      if(!content_resp.ok) { alert("Failed to load content"); return; }
      content_json = await content_resp.json();
    } else {
      content_json = content;
    }
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
    this.emulator.bus.register("emulator-ticked", () => this.replay_tick());
    // first time it runs, play_replay_slot 0 if movie is used
    if(movie) {
      const start_initial_replay = () => {
        this.emulator!.remove_listener("emulator-started", start_initial_replay);
        this.play_replay_slot(0);
      };
      this.emulator.add_listener("emulator-started", start_initial_replay);
    }
  }
}
function nonnull(obj:number|object|null):asserts obj {
  if(obj == null) {
    throw "Must be non-null";
  }
}
function setup_image(img:"bios"|"vga_bios"|"fda"|"fdb"|"hda"|"hdb"|"cdrom", content_json:ConfigSettings, config:V86StarterConfig, content_folder:string) {
  if(img in content_json) {
    const cjimg = content_json[img]!;
    if("url" in cjimg) {
      cjimg["url"] = content_folder+"/"+cjimg["url"]!;
      if("async" in cjimg) {
        // nop
      } else {
        cjimg["async"] = false;
      }
    }
    config[img] = cjimg;
  }
}
