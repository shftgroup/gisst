import { ipcRenderer } from 'electron';

export async function run_retroarch(core:string, content:string, entryState:bool, movie:bool) {
  ipcRenderer.send('gisst:run',core,content,entryState,movie);
}

export interface SavefileInfo {
  file:string;
}
export interface StatefileInfo {
  file:string;
  thumbnail_png_b64:string
}

export function on_saves_changed(f:(IpcRenderEvent, SavefileInfo) => void) {
  ipcRenderer.on('gisst:saves_changed',f);
}
export function on_states_changed(f:(IpcRenderEvent, StatefileInfo) => void) {
  ipcRenderer.on('gisst:states_changed',f);
}

export async function load_state(state_num:number) {
  ipcRenderer.send('gisst:load_state',state_num);
}
export async function download_file(category:"state"|"save"|"movie",file_name:string) {
  ipcRenderer.send('gisst:download_file',category, file_name);
}
