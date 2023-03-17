import {UI} from 'gisst-player';
import {saveAs} from './util';
import {Replay,Evt} from './v86replay';

let ui_state:UI;
let emulator:V86Starter;
// TODO wrap in a State class that includes the current replay ID if any
let states:ArrayBuffer[] = [];
let replays:Replay[] = [];
let active_replay:number|null = null;
export async function init(content_folder:string, content:string, entry_state:boolean, movie:boolean) {
  ui_state = new UI(
    <HTMLDivElement>document.getElementById("states")!,
    <HTMLDivElement>document.getElementById("replays")!,
    <HTMLDivElement>document.getElementById("saves")!,
    {
      "load_state":(num:number) => load_state_slot(num),
      "play_replay":(num:number) => play_replay_slot(num),
      "download_file":(category:"state" | "save" | "replay", file_name:string) => {
        if(category == "state") {
          const num_str = (file_name.match(/state([0-9]+)$/)?.[1]) ?? "0";
          const save_num = parseInt(num_str,10);
          saveAs(new Blob([states[save_num]]), file_name.toString()+".v86state");
        } else if(category == "save") {
          console.error("Not yet implemented");
        } else if(category == "replay") {
          const num_str = (file_name.match(/replay([0-9]+)$/)?.[1]) ?? "0";
          const replay_num = parseInt(num_str,10);
          saveAs(new Blob([replays[replay_num]]), file_name.toString()+".v86replay");
        } else {
          console.error("Invalid save category",category,file_name);
        }
      }
    }
  );

  document.getElementById("v86_controls")!.classList.remove("hidden");
  (document.getElementById("v86_save")!).onclick = async function() {
    const screenshot = emulator.screen_make_screenshot();
    ui_state.newState("state"+states.length.toString(), screenshot.src);
    states.push(await emulator.save_state());
  };
  (document.getElementById("v86_record")!).onclick = async function() {
    //const screenshot = emulator.screen_make_screenshot();
    if(active_replay != null) {
      await replays[active_replay!].stop(emulator);
    }
    ui_state.newReplay("replay"+replays.length.toString());
    active_replay = replays.length;
    replays.push(await Replay.start_recording(emulator));
  };
  (document.getElementById("v86_stop")!).onclick = async function() {
    if(active_replay != null) {
      await replays[active_replay].stop(emulator);
    }
  };
  //const content_base = content.substring(0, content.lastIndexOf("."));
  const config:V86StarterConfig = {
    wasm_path: "v86/v86.wasm",
    screen_container: <HTMLDivElement>document.getElementById("canvas_div")!,
    autostart: true
  };
  if(entry_state) {
    config["initial_state"] = {url:content_folder+"/entry_state"}
  }
  if(movie) {
    // do nothing for now
  }
  const content_resp = await fetch(content_folder+"/"+content);
  if(!content_resp.ok) { alert("Failed to load content"); return; }
  const content_json = await content_resp.json();
  setup_image("bios", content_json, config, "v86/bios");
  setup_image("vga_bios", content_json, config, "v86/bios");
  setup_image("fda", content_json, config, content_folder);
  setup_image("fdb", content_json, config, content_folder);
  setup_image("hda", content_json, config, content_folder);
  setup_image("hdb", content_json, config, content_folder);
  setup_image("cdrom", content_json, config, content_folder);
  let prev = document.getElementById("webplayer-preview")!;
  prev.classList.add("loaded");
  prev.addEventListener(
    "click",
    async function () {
      let canv = <HTMLCanvasElement>document.getElementById("canvas")!;
      prev.classList.add("hidden");
      document.getElementById("webplayer-textmode")!.classList.remove("hidden");
      emulator = new V86Starter(config);
      emulator.emulator_bus.register("keyboard-code", (k:number) => replay_log(emulator, Evt.KeyCode,k));
      emulator.emulator_bus.register("mouse-click", (v:[boolean,boolean,boolean]) => replay_log(emulator, Evt.MouseClick,v));
      emulator.emulator_bus.register("mouse-delta", (delta:[number,number]) => replay_log(emulator, Evt.MouseDelta,delta));
      emulator.emulator_bus.register("mouse-absolute", (pos:[number,number,number,number]) => replay_log(emulator, Evt.MouseAbsolute,pos));
      emulator.emulator_bus.register("mouse-wheel", (delta:[number,number]) => replay_log(emulator, Evt.MouseWheel, delta));
      emulator.bus.register("emulator-ticked", (_k:any) => replay_tick(emulator));
      canv.classList.remove("hidden");
      console.log(emulator.is_running());
      return false;
    });
}

function replay_log(emulator:V86Starter, evt:Evt, val:any) {
  if(active_replay != null) {
    replays[active_replay].log_evt(emulator, evt, val);
  }
}
function replay_tick(emulator:V86Starter) {
  if(active_replay != null) {
    replays[active_replay].tick(emulator);
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

async function load_state_slot(n:number) {
  if(active_replay != null) {
    console.log("loading states during replay recording/playback is not yet supported for v86");
    await replays[active_replay].stop(emulator);
  }
  emulator.restore_state(states[n]);
}
async function play_replay_slot(n:number) {
  if(active_replay != null) {
    await replays[active_replay].stop(emulator);
  }
  active_replay = n;
  await replays[n].start_playback(emulator);
}
