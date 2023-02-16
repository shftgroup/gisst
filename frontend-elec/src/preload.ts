// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

import { contextBridge, ipcRenderer } from 'electron';
import * as api from './api'

console.log(api);
contextBridge.exposeInMainWorld("api", api);
