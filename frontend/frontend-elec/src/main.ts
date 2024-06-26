import { app, BrowserWindow, ipcMain, IpcMainEvent, net } from 'electron';
import * as fs from 'fs';
import * as path from 'path';
import * as proc from 'child_process';
import * as url from 'url';
import * as ra_util from 'ra-util';
import {StateStart, ReplayStart, ColdStart, ObjectLink} from './types';
declare const MAIN_WINDOW_VITE_DEV_SERVER_URL: string;
declare const MAIN_WINDOW_VITE_NAME: string;

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) {
  app.quit();
}

if (process.defaultApp) {
  if (process.argv.length >= 2) {
    app.setAsDefaultProtocolClient('gisst', process.execPath, [path.resolve(process.argv[1])])
  }
} else {
  app.setAsDefaultProtocolClient('gisst')
}
let mainWindow:BrowserWindow;
const cache_dir = path.join(app.getPath("temp"),"gisst");
const resource_dir = path.resolve(__dirname, 'resources');
const config_dir = app.getPath("userData");
const content_dir = path.join(cache_dir, "content");
const saves_dir = path.join(cache_dir, "saves");
const states_dir = path.join(cache_dir, "states")
const createWindow = async (): Promise<void> => {
  //app.setAsDefaultProtocolClient(protocol[, path, args]);

  fs.mkdirSync(path.join(cache_dir, "core-options"), {recursive:true});
  fs.mkdirSync(path.join(cache_dir, "cache"), {recursive:true});
  fs.mkdirSync(path.join(cache_dir, "screenshots"), {recursive:true});
  fs.mkdirSync(path.join(config_dir, "remaps"), {recursive:true});

  fs.rmSync(content_dir, {recursive:true,force:true});

  let cfg = fs.readFileSync(path.join(resource_dir, 'ra-config-base.cfg'), {encoding:"utf8"});
  cfg = cfg.replace(/\$RESOURCE/g, resource_dir);
  cfg = cfg.replace(/\$CACHE/g, cache_dir);
  cfg = cfg.replace(/\$CONFIG/g, config_dir);
  fs.writeFileSync(path.join(cache_dir, "retroarch.cfg"),cfg);

  const is_darwin = process.platform === "darwin";
  if(is_darwin) {
    fs.chmodSync(path.join(resource_dir,"binaries","RetroArch.app", "Contents", "MacOS", "RetroArch"),"777");
  } else {
    if(fs.existsSync(path.join(resource_dir,"binaries","retroarch"))) {
      fs.chmodSync(path.join(resource_dir,"binaries","retroarch"),"777");
    }
    if(fs.existsSync(path.join(resource_dir,"binaries","RetroArch.AppImage"))) {
      fs.chmodSync(path.join(resource_dir,"binaries","RetroArch.AppImage"),"777");
    }
  }

  ipcMain.on('gisst:update_checkpoints', handle_update_checkpoints);
  ipcMain.on('gisst:run_retroarch', handle_run_retroarch);
  ipcMain.on('gisst:load_state', handle_load_state);
  ipcMain.on('gisst:load_checkpoint', handle_load_checkpoint);
  ipcMain.on('gisst:play_replay', handle_play_replay);
  ipcMain.on('gisst:download_file',handle_download_file);
  
  // Create the browser window.
  mainWindow = new BrowserWindow({
    height: 600,
    width: 800,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
    },
  });

  // and load the index.html of the app.
  console.log(MAIN_WINDOW_VITE_DEV_SERVER_URL, MAIN_WINDOW_VITE_NAME);
  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    await mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL);
  } else {
    await mainWindow.loadFile(path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`));
  }

  // Open the DevTools.
  mainWindow.webContents.openDevTools();
};

if(process.platform==="darwin") {
  // Some APIs can only be used after this event occurs.
  app.whenReady().then(() => 
    createWindow()
  ).then(() => 
    // if there's a CLI argument, run_url it
    run_cli()
  )

  // Handle the protocol. In this case, we choose to show an Error Box.
  app.on('open-url', (event, url) => {
    run_url(url)
  })
} else {
  const gotTheLock = app.requestSingleInstanceLock()

  if (!gotTheLock) {
    app.quit()
  } else {
    app.on('second-instance', (event, commandLine, _workingDirectory) => {
      // Someone tried to run a second instance, we should focus our window.
      if (mainWindow) {
        if (mainWindow.isMinimized()) mainWindow.restore()
        mainWindow.focus()
      }
      run_url(commandLine.pop());
    })

    // Create mainWindow, load the rest of the app, etc...
    app.whenReady().then(() => 
      createWindow()
    ).then(() => 
      // if there's a CLI argument, run_url it
      run_cli()
    )
  }
}

function run_cli() {
  if (process.argv.length > 1) {
    return run_url(process.argv[1]);
  }
}

function run_url(url:string) {
  console.log("main run URL",url);
  mainWindow!.webContents.send("gisst:handle_url",url);
}

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  // On OS X it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});
let current_replay:Replay | null = null;
let save_listener:fs.FSWatcher = null;
let state_listener:fs.FSWatcher = null;
let ra:proc.ChildProcess = null;
let seenSaves:string[] = [];
let seenReplays:Record<string,string> = {};
let seenCheckpoints:string[] = [];
let seenStates:Record<string, Buffer> = {};
async function handle_run_retroarch(evt:IpcMainEvent, host:string, core:string,start:ColdStart|StateStart|ReplayStart, manifest:ObjectLink[]) {
  seenStates = {};
  seenSaves = [];
  seenReplays = {};
  seenCheckpoints = [];
  const content = manifest.find((o) => o.object_role=="content")!;
  const content_file = content.file_filename!;
  const dash_point = content_file.indexOf("-");
  const content_base = content_file.substring(dash_point < 0 ? 0 : dash_point, content_file.lastIndexOf("."));
  const entryState = start.type == "state";
  const replay = start.type == "replay";
  console.assert(!(entryState && replay), "It is invalid to have both an entry state and play back a replay");
  clear_current_replay(evt);
  const retro_args = ["-v", "-c", path.join(cache_dir, "retroarch.cfg"), "--appendconfig", path.join(content_dir, "retroarch.cfg"), "-L", core];
  if (entryState) {
    retro_args.push("-e");
    retro_args.push("1");
  }
  if (replay) {
    retro_args.push("-P");
    retro_args.push(path.join(states_dir, content_base+".replay1"));
  } else {
    retro_args.push("-R");
    retro_args.push(path.join(states_dir, content_base+".replay1"));
  }
  retro_args.push(path.join(content_dir, content.file_filename));
  console.log(retro_args);

  if(save_listener != null) {
    save_listener.close();
    save_listener = null;
  }
  if(state_listener != null) {
    state_listener.close();
    state_listener = null;
  }

  fs.rmSync(saves_dir, {recursive:true,force:true});
  fs.rmSync(states_dir, {recursive:true,force:true});
  fs.rmSync(content_dir, {recursive:true,force:true});
  fs.mkdirSync(saves_dir, {recursive:true});
  fs.mkdirSync(states_dir, {recursive:true});
  fs.mkdirSync(content_dir, {recursive:true});
  
  let proms = [];
  // copy all files from manifest
  for(const file of manifest) {
    proms.push(get_storage_file(host, file.file_dest_path, file.file_hash, file.file_filename, path.join(content_dir, file.file_source_path)));
  }
  await Promise.all(proms);
  proms = null;

  save_listener = fs.watch(saves_dir, {"persistent":false}, function(file_evt, name) {
    console.log("saves",file_evt,name);
    if(!seenSaves.includes(name)) {
      seenSaves.push(name);
      evt.sender.send('gisst:saves_changed', {
        "file":name,
      });
    }
  });
  state_listener = fs.watch(states_dir, {"persistent":false}, function(file_evt, name) {
    console.log("states",file_evt,name);
    if(name.endsWith(".png")) {
      const img_path = path.join(states_dir,name);
      console.log("img",img_path,fs.statSync(img_path));
      const file_name = name.substring(0, name.length-4);
      if(file_name in seenStates) {
        console.log("seen image already");
        return;
      }
      if(fs.statSync(img_path).size == 0) {
        console.log("image file is empty");
        return;
      }
      console.log("image ready, send along",fs.statSync(img_path));
      const image_data = fs.readFileSync(img_path);
      seenStates[file_name] = image_data;
      // If it's already a known checkpoint or state, ignore it
      if(!seenCheckpoints.includes(file_name)) {
        const replay = ra_util.replay_of_state((fs.readFileSync(path.join(states_dir,file_name))));
        let known_replay = false;
        if(replay) {
          for(const seen in seenReplays) {
            if(seenReplays[seen] == replay.id) {
              known_replay = true;
            }
          }
        }
        if(replay && current_replay && replay.id == current_replay.id) {
          seenCheckpoints.push(file_name);
          // add it as a mere checkpoint if it's associated with a replay
          evt.sender.send('gisst:replay_checkpoints_changed', {
            "added":[{"file": file_name,"thumbnail": image_data.toString('base64'),}],
            "delete_old":false
          });
          // otherwise ignore it if it's a checkpoint from a non-current replay we have locally
        } else if(!replay || !known_replay) {
          evt.sender.send('gisst:states_changed', {
            "file": file_name,
            "thumbnail": image_data.toString('base64')
          });
        }
      }
    }
    if(name.includes(".replay")) {
      const replay_path = path.join(states_dir,name);
      console.log("replay",replay_path,fs.statSync(replay_path));
      if(name in seenReplays) {
        console.log("seen replay already");
        return;
      }
      if(fs.statSync(replay_path).size == 0) {
        console.log("replay file is empty");
        return;
      }
      console.log("replay ready, send along",fs.statSync(replay_path));
      seenReplays[name] = ra_util.replay_info(new Uint8Array(fs.readFileSync(replay_path).buffer)).id;
      evt.sender.send('gisst:replays_changed', {
        "file": name,
      });
    }
  });

  if (entryState) {
    // Cast: This one is definitely a statestart because the type is state
    const data = (start as StateStart).data;
    await get_storage_file(host, data.file_dest_path, data.file_hash, data.file_filename, path.join(content_dir, "entry_state"));
    fs.copyFileSync(path.join(content_dir, "entry_state"), path.join(cache_dir, "states", content_base+".state1.entry"));
    fs.copyFileSync(path.join(content_dir, "entry_state"), path.join(cache_dir, "states", content_base+".state1"));
    if(fs.existsSync(path.join(content_dir, "entry_state.png"))){
      fs.copyFileSync(path.join(content_dir,"entry_state.png"), path.join(cache_dir, "states", content_base+".state1.png"));
    } else {
      fs.copyFileSync(path.join(resource_dir,"init_state.png"), path.join(cache_dir, "states", content_base+".state1.png"));
    }
  }
  if (replay) {
    const data = (start as ReplayStart).data;
    await get_storage_file(host, data.file_dest_path, data.file_hash, data.file_filename, path.join(content_dir, "replay.replay"));
    fs.copyFileSync(path.join(content_dir, "replay.replay"), path.join(cache_dir, "states", content_base+".replay1"));
  } else {
    const f = fs.openSync(path.join(cache_dir, "states", content_base+".replay1"), 'w');
    fs.writeSync(f, Buffer.from("\0"), 0, 1);
    fs.closeSync(f);
  }

  const is_darwin = process.platform === "darwin";
  let binary;

  if(is_darwin) {
    binary = path.join(resource_dir,"binaries","RetroArch.app", "Contents", "MacOS", "RetroArch");
  } else {
    let bin_name = "RetroArch.AppImage";
    if(fs.existsSync(path.join(resource_dir,"binaries","retroarch"))) {
      bin_name="retroarch";
    }
    binary = path.join(resource_dir,"binaries",bin_name);
  }
  if(ra != null) {
    ra.unref();
    ra.kill();
  }
  ra = proc.spawn(binary, retro_args, {"windowsHide":true,"detached":false});
  ra.stderr.on('data', (data) => console.log("err",data.toString()));
  // ra.stdout.on('data', (data) => console.log("out",data.toString()));
  ra.on('close', (exit_code) => console.log("exit",exit_code));
  ra.on('error', (error) => console.error("failed to start RA",error));
  ra.stdout.resume();
  const prom = send_message("SAVE_STATE");
  await prom;
}

async function read_response(wait:boolean): Promise<string | null> {
  const waiting = () => new Promise((resolve,_reject) => {
    const read_cb = () => {
      ra.stdout.removeListener("readable",read_cb);
      resolve(null);
    }
    ra.stdout.on("readable", read_cb);
  });
  if(wait) {
    await waiting();
  }
  let more = ra.stdout.read();
  let outp = more ? "" : more;
  while(more) {
    outp += more;
    more = ra.stdout.read();
  }
  console.log("stdout: ",outp);
  return outp;
}
enum ReplayMode {
  Record,
  Playback,
  Inactive
}
interface Replay {
  finished:boolean;
  mode:ReplayMode;
  id:string;
}
async function handle_play_replay(evt:IpcMainEvent, num:number) {
  clear_current_replay(evt);
  send_message("PLAY_REPLAY_SLOT "+num);
  const resp = await read_response(true);
  nonnull(resp);
  const num_str = (resp.match(/PLAY_REPLAY_SLOT ([0-9]+)$/)?.[1]) ?? "0";
  if(num_str == "0") {
    return;
  }
  current_replay = {mode:ReplayMode.Playback,id:num_str,finished:false};
  find_checkpoints_inner(evt);
}
async function handle_load_state(evt:IpcMainEvent, num:number) {
  send_message("LOAD_STATE_SLOT "+num.toString());
}
async function handle_load_checkpoint(evt:IpcMainEvent, num:number) {
  send_message("LOAD_STATE_SLOT "+num.toString());
}
enum BSVFlags {
  START_RECORDING    = (1 << 0),
  START_PLAYBACK     = (1 << 1),
  PLAYBACK           = (1 << 2),
  RECORDING          = (1 << 3),
  END                = (1 << 4),
  EOF_EXIT           = (1 << 5)
}
async function send_message(msg:string) {
  let clearout = await read_response(false);
  while(clearout) { clearout = await read_response(false); }
  console.log("send:",msg);
  ra.stdin.write(msg+"\n");
}
// Called by IPC from time to time
async function handle_update_checkpoints(evt:IpcMainEvent) {
  console.log("Update cps/replay status");
  await send_message("GET_CONFIG_PARAM active_replay");
  const resp = await read_response(true);
  nonnull(resp);
  const matches = resp.match(/GET_CONFIG_PARAM active_replay ([0-9]+) ([0-9]+)$/);
  const id = (matches?.[1]) ?? "0";
  const flags = parseInt((matches?.[2]) ?? "0",10);
  if(id == "0" || flags == 0) {
    console.log("no current replay or different replay started");
    clear_current_replay(evt);
  } else {
    if(current_replay && current_replay.id != id) {
      clear_current_replay(evt);
    }
    const finished = (flags & BSVFlags.END) != 0;
    const mode = (flags & BSVFlags.PLAYBACK) != 0 ? ReplayMode.Playback : (flags & BSVFlags.RECORDING ? ReplayMode.Record : ReplayMode.Inactive);
    console.log("current replay",id,mode,finished);
    current_replay = {id:id,mode:mode,finished:finished};
  }
  if(current_replay) {
    find_checkpoints_inner(evt);
  }
}
function clear_current_replay(evt:IpcMainEvent) {
  current_replay = null;
  seenCheckpoints = [];
  evt.sender.send('gisst:replay_checkpoints_changed', {
    "added":[],
    "delete_old":true
  });
}

function find_checkpoints_inner(evt:IpcMainEvent) {
  // search state files for states saved of current replay
  console.log(seenStates);
  for(const state_file in seenStates) {
    if(seenCheckpoints.includes(state_file)) { continue; }
    console.log("Check ",state_file);
    const buf = fs.readFileSync(path.join(states_dir,state_file)).buffer;
    if(buf.byteLength < 8) {
      console.log("State file is too small");
      continue;
    }
    const replay = ra_util.replay_of_state(new Uint8Array(buf));
    console.log("Replay info",replay,"vs",current_replay);
    if(replay && replay.id == current_replay.id) {
      seenCheckpoints.push(state_file);
      evt.sender.send('gisst:replay_checkpoints_changed', {
        "added":[{"file": state_file,"thumbnail": seenStates[state_file].toString('base64')}],
        "delete_old":false
      });
    }
  }
}
function handle_download_file(evt:IpcMainEvent, category:"save"|"state"|"replay", file_name:string) {
  try{
    let fpath;
    if(category == "state") {
      fpath = states_dir;
    } else if(category == "save") {
      fpath = saves_dir;
    } else if(category == "replay") {
      fpath = states_dir;
    } else {
      console.error("Invalid save category",category,file_name);
    }
    fpath = path.join(fpath,file_name);
    evt.sender.downloadURL(url.pathToFileURL(fpath).toString());
  }
  catch(e) {console.error(e); throw e;}
}
function nonnull(obj:string|object|null):asserts obj {
  if(obj == null) {
    throw "Must be non-null";
  }
}
async function get_storage_file(host:string, storage_base:string, hash:string, filename:string, local_dest:string) {
  const resp = await net.fetch(host+"/storage/"+storage_base+"/"+hash+"-"+filename,{"mode":"no-cors"});
  const bytes = await resp.arrayBuffer();
  console.log(host,storage_base,hash,filename,resp,local_dest,bytes.byteLength);
  await fs.writeFileSync(local_dest, new Uint8Array(bytes));
}
