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
import {UI} from 'gisst-player';

let ui_state;
let active_core = null;

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

let v86_emulator = null;
async function v86_save_state(state_num:number) {
  const screenshot = v86_emulator.screen_make_screenshot();
  ui_state.newState("state"+v86_states.length.toString(), screenshot.src);
  v86_states.push(await v86_emulator.save_state());
}
function v86_load_state(state_num:number) {
  v86_emulator.restore_state(v86_states[state_num]);
}
function v86_play_replay(replay_num:number) {
  console.error("not yet implemented");
}
function v86_download_file(category:"state"|"save"|"replay",file_name:string) {
  if(category == "state") {
    const num_str = (file_name.match(/state([0-9]+)$/)?.[1]) ?? "0";
    const save_num = parseInt(num_str,10);
    saveAs(new Blob([v86_states[save_num]]), file_name.toString()+".v86state");
  } else if(category == "save") {
    console.error("Not yet implemented");
  } else if(category == "replay") {
    console.error("Not yet implemented");
  } else {
    console.error("Invalid save category",category,file_name);
  }
}
function saveAs(blob:Blob, name:string) {
    // Namespace is used to prevent conflict w/ Chrome Poper Blocker extension (Issue https://github.com/eligrey/FileSaver.js/issues/561)
  const a = <HTMLAnchorElement>document.createElementNS('http://www.w3.org/1999/xhtml', 'a');
  a.download = name;
  a.rel = 'noopener';
  a.href = URL.createObjectURL(blob);

  setTimeout(() => URL.revokeObjectURL(a.href), 40 /* sec */ * 1000);
  setTimeout(() => a.click(), 0);
}

let v86_states = [];
async function run_v86(content:string, entryState:bool, movie:bool) {
  const content_folder = "renderer-resources/content";
  const config:V86StarterConfig = {
    wasm_path: "renderer-resources/v86/v86.wasm",
    screen_container: <HTMLDivElement>document.getElementById("v86-container")!,
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
  setup_image("bios", content_json, config, "renderer-resources/v86/bios");
  setup_image("vga_bios", content_json, config, "renderer-resources/v86/bios");
  setup_image("fda", content_json, config, content_folder);
  setup_image("fdb", content_json, config, content_folder);
  setup_image("hda", content_json, config, content_folder);
  setup_image("hdb", content_json, config, content_folder);
  setup_image("cdrom", content_json, config, content_folder);

  v86_emulator = new V86Starter(config);
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

async function run(core:string, content:string, entryState:bool, movie:bool) {
  ui_state.clear();
  active_core = core;
  v86_states = [];
  if(v86_emulator) {
    v86_emulator.destroy();
    v86_emulator = null;
  }
  v86_emulator = null;
  if(core == "v86") {
    (document.getElementById("v86-container")!).classList.remove("hidden");
    (document.getElementById("v86-controls")!).classList.remove("hidden");
    // This one operates entirely within the renderer side of things
    run_v86(content, entryState, movie);
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
  document.getElementById("v86_save")?.addEventListener("click", v86_save_state);
  ui_state = new UI(document.getElementById("states")!, document.getElementById("replays")!, document.getElementById("saves")!, {
    "load_state":(number) => {if (active_core == "v86") { v86_load_state(number); } else { api.load_state(number); }},
    "play_replay":(number) => {if (active_core == "v86") { v86_play_replay(number); } else { api.play_replay(number); }},
    "download_file":(category:"state" | "save" | "replay", file_name:string) => {if (active_core == "v86") { v86_download_file(category, file_name); } else { api.download_file(category, file_name); }},
  });
});
