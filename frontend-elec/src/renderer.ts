/**
 * This file will automatically be loaded by webpack and run in the "renderer" context.
 * To learn more about the differences between the "main" and the "renderer" context in
 * Electron, visit:
 *
 * https://electronjs.org/docs/latest/tutorial/process-model
 *
 * By default, Node.js integration in this file is disabled. When enabling Node.js integration
 * in a renderer process, please be aware of potential security implications. You can read
 * more about security risks here:
 *
 * https://electronjs.org/docs/tutorial/security
 *
 * To enable Node.js integration in this file, open up `main.js` and enable the `nodeIntegration`
 * flag:
 *
 * ```
 *  // Create the browser window.
 *  mainWindow = new BrowserWindow({
 *    width: 800,
 *    height: 600,
 *    webPreferences: {
 *      nodeIntegration: true
 *    }
 *  });
 * ```
 */

import './index.css';
import {IpcRendererEvent} from 'electron';
import {UI} from 'gisst-player';
import {EmbedV86,StateInfo} from 'embedv86';
import {saveAs} from './util';
import {SavefileInfo, StatefileInfo, ReplayfileInfo, ReplayCheckpointInfo, Environment, ColdStart, StateStart, ReplayStart, ObjectLink} from './types';

let ui_state:UI;
let active_core:string|null = null;
const host = "http://localhost:3000";
let v86:EmbedV86 = new EmbedV86({
  wasm_root:"renderer-resources/v86",
  bios_root:"renderer-resources/v86/bios",
  content_root:host,
  container: <HTMLDivElement>document.getElementById("v86-container")!,
  register_replay:(nom:string)=>ui_state.newReplay(nom),
  stop_replay:()=>{
    ui_state.clearCheckpoints();
  },
  states_changed:(added:StateInfo[], removed:StateInfo[]) => {
    for(let si of removed) {
      ui_state.removeState(si.name);
    }
    for(let si of added) {
      ui_state.newState(si.name,si.thumbnail);
    }
  },
  replay_checkpoints_changed:(added:StateInfo[], removed:StateInfo[]) => {
    for(let si of removed) {
      ui_state.removeCheckpoint(si.name);
    }
    for(let si of added) {
      ui_state.newCheckpoint(si.name,si.thumbnail);
    }
  },
});

function nested_replace(obj:any, target:string, replacement:string) {
  for(let key in obj) {
    if(obj[key] == target) {
      obj[key] = replacement;
    } else if(typeof(obj[key]) == "object") {
      nested_replace(obj[key], target, replacement);
    }
  }
}

function init_v86(environment:Environment, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[]) {
  let content = manifest.find((o) => o.object_role=="content")!;
  let content_path = "storage/"+content.object_dest_path+"/"+content.object_hash+"-"+content.object_filename;
  nested_replace(environment.environment_config, "$CONTENT", content_path);
  let entry_state:string|null = null;
  if (start.type == "state") {
    let data = (start as StateStart).data;
    entry_state = "storage/"+data.state_path+"/"+data.state_hash+"-"+data.state_filename;
  }
  let movie:string|null = null;
  if (start.type == "replay") {
    let data = (start as ReplayStart).data;
    movie = "storage/"+data.replay_path+"/"+data.replay_hash+"-"+data.replay_filename;
  }
  v86.run(environment.environment_config, entry_state, movie);
}

function saves_updated(evt:IpcRendererEvent, saveinfo:SavefileInfo) {
  console.log("new save",saveinfo);
  ui_state.newSave(saveinfo.file);
}
api.on_saves_changed(saves_updated);
function states_updated(evt:IpcRendererEvent, stateinfo:StatefileInfo) {
  console.log("states", evt, stateinfo);
  ui_state.newState(stateinfo.file,stateinfo.thumbnail);
}
api.on_states_changed(states_updated);
function replays_updated(evt:IpcRendererEvent, replayinfo:ReplayfileInfo) {
  console.log("replays", evt, replayinfo);
  ui_state.newReplay(replayinfo.file);
}
api.on_replays_changed(replays_updated);
function checkpoints_updated(evt:IpcRendererEvent, info:ReplayCheckpointInfo) {
  console.log("checkpoints", evt, info);
  if(info.delete_old) {
    ui_state.clearCheckpoints();
  }
  for(let check of info.added) {
    ui_state.newCheckpoint(check.file, check.thumbnail);
  }
}
api.on_replay_checkpoints_changed(checkpoints_updated);

async function run(content:string, entryState:string, movie:string) {
  ui_state.clear();
  let data_resp = await fetch(host+"/play/"+content+(entryState ? "?state="+entryState : "")+(movie ? "?replay="+movie : ""), {headers:[["Accept","application/json"]]});
  console.log(data_resp);
  let config = await data_resp.json();
  console.log(config);
  let core_kind = config.environment.environment_framework;
  v86.clear();
  if(core_kind == "v86") {
    active_core = "v86";
    (document.getElementById("v86-container")!).classList.remove("hidden");
    (document.getElementById("v86-controls")!).classList.remove("hidden");
    // This one operates entirely within the renderer side of things
    init_v86(config.environment, config.start, config.manifest);
  } else {
    active_core = config.environment.environment_core_name;
    (document.getElementById("v86-controls")!).classList.add("hidden");
    (document.getElementById("v86-container")!).classList.add("hidden");
    api.run_retroarch(active_core, config.start, config.manifest);
  }
}

function bootCold() {
  let instance = <HTMLInputElement>document.getElementById("boot-instance")!;
  run(instance.value, null, null);
}
function bootState() {
  let instance = <HTMLInputElement>document.getElementById("boot-instance")!;
  let state = <HTMLInputElement>document.getElementById("boot-state")!;
  run(instance.value, state.value, null);
}
function bootReplay() {
  let instance = <HTMLInputElement>document.getElementById("boot-instance")!;
  let replay = <HTMLInputElement>document.getElementById("boot-replay")!;
  run(instance.value, null, replay.value);
}

window.addEventListener("DOMContentLoaded", () => {
  document.getElementById("boot-cold-button")?.addEventListener("click",
    () => bootCold()
  );
  document.getElementById("boot-state-button")?.addEventListener("click",
    () => bootState()
  );
  document.getElementById("boot-replay-button")?.addEventListener("click",
    () => bootReplay()
  );
  document.getElementById("v86_save")?.addEventListener("click",
    () => v86.save_state()
  );
  document.getElementById("v86_record")?.addEventListener("click",
    () => v86.record_replay()
  );
  document.getElementById("v86_halt")?.addEventListener("click",
    () => v86.stop_replay()
  );
  setInterval(() => {
    if(active_core && active_core != "v86") {
      api.update_checkpoints();
    }
  }, 1000);
  ui_state = new UI(
    <HTMLDivElement>document.getElementById("ui")!,
    {
      "load_state":(n:number) => {
        if (active_core == "v86") {
          if(v86.active_replay != null) { v86.stop_replay(); }
          v86.load_state_slot(n);
        } else {
          api.load_state(n);
        }
      },
      "play_replay":(n:number) => {
        if (active_core == "v86") {
          v86.play_replay_slot(n);
        } else {
          api.play_replay(n);
        }
      },
      "load_checkpoint":(n:number) => {
        if (active_core == "v86") {
          if(v86.active_replay == null) { throw "Can't load checkpoint if no replay"; }
          v86.load_state_slot(n);
        } else {
          api.load_checkpoint(n);
        }
      },
      "download_file":(category:"state" | "save" | "replay", file_name:string) => {
        if (active_core == "v86") {
          v86.download_file(category, file_name).then(([blob,name]) => saveAs(blob,name));
        } else {
          api.download_file(category, file_name);
        }
      },
  });
});
