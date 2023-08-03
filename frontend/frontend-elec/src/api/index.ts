import { ipcRenderer, IpcRendererEvent } from 'electron';
import {StartStateData, StateStart, StartReplayData, ReplayStart, ColdStart, ObjectLink, SavefileInfo, StatefileInfo, ReplayfileInfo, ReplayCheckpointInfo} from '../types';

export async function run_retroarch(host:string, core:string, start:ColdStart|StateStart|ReplayStart, manifest:ObjectLink[]) {
  ipcRenderer.send('gisst:run_retroarch',host,core,start,manifest);
}

export function on_handle_url(f:(evt:IpcRendererEvent, url:string) => void) {
  ipcRenderer.on('gisst:handle_url',f);
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
