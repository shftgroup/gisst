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
import {EmbedV86} from 'embedv86';
import {saveAs} from './util';
import {SavefileInfo, StatefileInfo, ReplayfileInfo} from './api';

let ui_state:UI;
let active_core:string|null = null;
let v86:EmbedV86 = new EmbedV86({wasm_root:"renderer-resources/v86",bios_root:"renderer-resources/v86/bios", content_root:"renderer-resources/content",container: <HTMLDivElement>document.getElementById("v86-container")!});

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

async function run(core:string, content:string, entryState:boolean, movie:boolean) {
  ui_state.clear();
  active_core = core;
  v86.clear();
  if(core == "v86") {
    (document.getElementById("v86-container")!).classList.remove("hidden");
    (document.getElementById("v86-controls")!).classList.remove("hidden");
    // This one operates entirely within the renderer side of things
    v86.run(content, entryState, movie);
  } else {
    (document.getElementById("v86-controls")!).classList.add("hidden");
    (document.getElementById("v86-container")!).classList.add("hidden");
    api.run_retroarch(core, content, entryState, movie);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  document
    .querySelector("#run-v86-button")
    ?.addEventListener("click", () => run("v86", "freedos722-root.json", false, false));
  document
    .querySelector("#run-cold-button")
    ?.addEventListener("click", () => run("fceumm", "bfight.nes", false, false));
  document
    .querySelector("#run-entry-button")
    ?.addEventListener("click", () => run("fceumm", "bfight.nes", true, false));
  document
    .querySelector("#run-movie-button")
    ?.addEventListener("click", () => run("fceumm", "bfight.nes", false, true));
  document.getElementById("v86_save")?.addEventListener("click",
    () => v86.save_state((nom:string, thumb:string) => ui_state.newState(nom,thumb))
  );
  document.getElementById("v86_record")?.addEventListener("click",
    () => v86.record_replay((nom:string)=>ui_state.newReplay(nom))
  );
  document.getElementById("v86_halt")?.addEventListener("click",
    () => v86.stop_replay()
  );
  ui_state = new UI(
    <HTMLDivElement>document.getElementById("states")!,
    <HTMLDivElement>document.getElementById("replays")!,
    <HTMLDivElement>document.getElementById("saves")!,
    {
      "load_state":(n:number) => {
        if (active_core == "v86") {
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
      "download_file":(category:"state" | "save" | "replay", file_name:string) => {
        if (active_core == "v86") {
          let [blob,name] = v86.download_file(category, file_name);
          saveAs(blob, name);
        } else {
          api.download_file(category, file_name);
        }
      },
  });
});
