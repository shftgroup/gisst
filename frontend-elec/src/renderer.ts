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
import {SavefileInfo, StatefileInfo, ReplayfileInfo, ReplayCheckpointInfo} from './api';

let ui_state:UI;
let active_core:string|null = null;
let v86:EmbedV86 = new EmbedV86({
  wasm_root:"renderer-resources/v86",
  bios_root:"renderer-resources/v86/bios",
  content_root:"renderer-resources/content",
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
  let data_resp = await fetch("http://localhost:3000/play/"+content+(entryState ? "?state="+entryState : "")+(movie ? "?replay="+movie : ""), {mode:"no-cors", headers:[["Accept","application/json"]]});
  console.log(data_resp);
  let config_data = await data_resp.json();
  let config = JSON.parse(config_data);
  console.log(config);
  let core_kind = config.instance.instance_framework;
  v86.clear();
  if(core_kind == "v86") {
    active_core = "v86";
    (document.getElementById("v86-container")!).classList.remove("hidden");
    (document.getElementById("v86-controls")!).classList.remove("hidden");
    // This one operates entirely within the renderer side of things
    //v86.run(config.environment, config.start, config.manifest);
  } else {
    active_core = config.environment.core_name;
    (document.getElementById("v86-controls")!).classList.add("hidden");
    (document.getElementById("v86-container")!).classList.add("hidden");
    api.run_retroarch(active_core, config.start, config.manifest);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  // document
  //   .querySelector("#run-v86-cold-button")
  //   ?.addEventListener("click", () => run("freedos722-root.json", false, false));
  // document
  //   .querySelector("#run-v86-entry-button")
  //   ?.addEventListener("click", () => run("freedos722-root.json", true, false));
  // document
  //   .querySelector("#run-v86-movie-button")
  //   ?.addEventListener("click", () => run("freedos722-root.json", false, true));
  document
    .querySelector("#run-cold-button")
    ?.addEventListener("click", () => run("00000000000000000000000000000000", null, null));
  document
    .querySelector("#run-entry-button")
    ?.addEventListener("click", () => run("00000000000000000000000000000000", "00000000000000000000000000000000", null));
  document
    .querySelector("#run-movie-button")
    ?.addEventListener("click", () => run("00000000000000000000000000000000", null, "00000000000000000000000000000000"));
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
