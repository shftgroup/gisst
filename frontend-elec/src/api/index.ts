import { ipcRenderer, IpcRendererEvent } from 'electron';

export async function run_retroarch(core:string, start:ColdStart|StateStart|ReplayStart, manifest:ObjectLink[]) {
  ipcRenderer.send('gisst:run_retroarch',core,start,manifest);
}

export interface ObjectLink {
  object_role:string,
  object_dest_path:string,
  object_filename:string,
  object_source_path:string,
  object_hash:string,
  object_id:string,
}

export interface ColdStart {
  type:string
}

export interface StartStateData {
  is_checkpoint:boolean,
  state_description:string,
  state_filename:string,
  state_hash:string,
  state_id:string,
  state_path:string
}

export interface StateStart {
  type:string,
  data:StartStateData
}

export interface StartReplayData {
  replay_filename:string,
  replay_hash:string,
  replay_id:string,
  replay_path:string
}

export interface ReplayStart {
  type:string,
  data:StartReplayData
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
export async function load_checkpoint(cp_num:number) {
  ipcRenderer.send('gisst:load_checkpoint',cp_num);
}
export async function play_replay(replay_num:number) {
  ipcRenderer.send('gisst:play_replay',replay_num);
}
export async function download_file(category:"state"|"save"|"replay",file_name:string) {
  ipcRenderer.send('gisst:download_file',category, file_name);
}
