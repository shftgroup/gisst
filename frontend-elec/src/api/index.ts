import { ipcRenderer, IpcRendererEvent } from 'electron';

export async function run_retroarch(core:string, content:string, entryState:boolean, movie:boolean) {
  ipcRenderer.send('gisst:run_retroarch',core,content,entryState,movie);
}

export interface SavefileInfo {
  file:string;
}
export interface StatefileInfo {
  file:string;
  thumbnail:string;
}
export interface ReplayfileInfo {
  file:string;
}
export interface ReplayCheckpointInfo {
  added:[StatefileInfo];
  delete_old?:boolean;
}

export function on_saves_changed(f:(evt:IpcRendererEvent, info:SavefileInfo) => void) {
  ipcRenderer.on('gisst:saves_changed',f);
}
export function on_states_changed(f:(evt:IpcRendererEvent, info:StatefileInfo) => void) {
  ipcRenderer.on('gisst:states_changed',f);
}
export function on_replays_changed(f:(evt:IpcRendererEvent, info:ReplayfileInfo) => void) {
  ipcRenderer.on('gisst:replays_changed',f);
}
export function on_replay_checkpoints_changed(f:(evt:IpcRendererEvent, info:ReplayCheckpointInfo) => void) {
  ipcRenderer.on('gisst:replay_checkpoints_changed',f);
}

export async function update_checkpoints() {
  ipcRenderer.send('gisst:update_checkpoints');
}
export async function load_state(state_num:number) {
  ipcRenderer.send('gisst:load_state',state_num);
}
export async function play_replay(replay_num:number) {
  ipcRenderer.send('gisst:play_replay',replay_num);
}
export async function download_file(category:"state"|"save"|"replay",file_name:string) {
  ipcRenderer.send('gisst:download_file',category, file_name);
}
