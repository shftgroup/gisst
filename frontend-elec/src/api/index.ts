import { ipcRenderer } from 'electron';

export async function run_retroarch(core:string, content:string, entryState:bool, movie:bool) {
  ipcRenderer.send('gisst:run',core,content,entryState,movie);
}

export interface SavefileInfo {
  file:string;
  thumbnail_png_b64:string
}
export interface StatefileInfo {
  file:string;
}

export function on_saves_changed(f:(IpcRenderEvent, SavefileInfo) => void) {
  ipcRenderer.on('gisst:saves_changed',f);
}
export function on_states_changed(f:(IpcRenderEvent, StateefileInfo) => void) {
  ipcRenderer.on('gisst:states_changed',f);
}
