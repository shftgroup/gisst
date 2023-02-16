import { app, ipcRenderer } from 'electron';
import * as path from 'path';

export async function run_retroarch(core:string, content:string, entryState:bool, movie:bool) {
  ipcRenderer.send('gisst:run',core,content,entryState,movie);
}

export function on_saves(f:Function[IpcRendererEvent]) {
  ipcRenderer.on('gisst:saves',f);
}
