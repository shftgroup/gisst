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

function saves_updated(evt:IpcRendererEvent, saveinfo:SavefileInfo) {
  console.log("new save",saveinfo);
  ui_state.newSave(saveinfo.file);
}
api.on_saves_changed(saves_updated);
function states_updated(evt:IpcRendererEvent, stateinfo:StatefileInfo) {
  console.log("states", evt, stateinfo);
  ui_state.newState(stateinfo.file,stateinfo.thumbnail_png_b64);
}
api.on_states_changed(states_updated);

async function run(core:string, content:string, entryState:bool, movie:bool) {
  ui_state.clear();
  api.run_retroarch(core, content, entryState, movie);
}

window.addEventListener("DOMContentLoaded", () => {
  document
    .querySelector("#run-cold-button")
    ?.addEventListener("click", () => run("fceumm", "bfight.nes", false, false));
  document
    .querySelector("#run-entry-button")
    ?.addEventListener("click", () => run("fceumm", "bfight.nes", true, false));
  document
    .querySelector("#run-movie-button")
    ?.addEventListener("click", () => run("fceumm", "bfight.nes", false, true));
  ui_state = new UI(document.getElementById("states")!, document.getElementById("saves")!, {
    "load_state":(number) => api.load_state(number),
  });
});
