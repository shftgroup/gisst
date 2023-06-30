import './style.css';
import IMG_STATE_ENTRY from './media/init_state.png';
import * as fetchfs from './fetchfs';
import {UI} from 'gisst-player';
import {saveAs,base64EncArr} from './util';
import * as ra_util from 'ra-util';

const FS_CHECK_INTERVAL = 1000;

const state_dir = "/home/web_user/retroarch/userdata/states";
const saves_dir = "/home/web_user/retroarch/userdata/saves";

const retro_args = ["-v"];

let ui_state:UI;

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

export function init(core:string, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[]) {
  let content = manifest.find((o) => o.object_role=="content")!;
  let content_file = content.object_filename!;
  let dash_point = content_file.indexOf("-");
  let content_base = content_file.substring(dash_point < 0 ? 0 : dash_point, content_file.lastIndexOf("."));
  const entryState = start.type == "state";
  const movie = start.type == "replay";
  if (entryState) {
    retro_args.push("-e");
    retro_args.push("1");
  }
  if (movie) {
    retro_args.push("-P");
    retro_args.push(state_dir+"/"+content_base+".replay1");
  } else {
    retro_args.push("-R");
    retro_args.push(state_dir+"/"+content_base+".replay1");
  }

  retro_args.push("--appendconfig");
  retro_args.push("/home/web_user/content/retroarch.cfg");
  retro_args.push("/home/web_user/content/" + content_file);
  console.log(retro_args);

  loadRetroArch(core,
    function () {
      fetchfs.mkdirp("/home/web_user/content");

      let proms = [];
      
      proms.push(fetchfs.registerFetchFS(("/assets/frontend/bundle/.index-xhr"), "/assets/frontend/bundle", "/home/web_user/retroarch/bundle", true));

      for(let file of manifest) {
        let file_prom = fetchfs.fetchFile("/storage/"+file.object_dest_path+"/"+file.object_hash+"-"+file.object_filename,"/home/web_user/content/"+file.object_source_path,true);
        proms.push(file_prom);
      }
      if (entryState) {
        // Cast: This one is definitely a statestart because the type is state
        let data = (start as StateStart).data;
        console.log(data, "/storage/"+data.state_path+"/"+data.state_hash+"-"+data.state_filename,"/home/web_user/content/entry_state");
        proms.push(fetchfs.fetchFile("/storage/"+data.state_path+"/"+data.state_hash+"-"+data.state_filename,"/home/web_user/content/entry_state",false));
      }
      if (movie) {
        // Cast: This one is definitely a replaystart because the type is state
        let data = (start as ReplayStart).data;
        console.log(data, "/storage/"+data.replay_path+"/"+data.replay_hash+"-"+data.replay_filename,"/home/web_user/content/replay.replay1");
        proms.push(fetchfs.fetchFile("/storage/"+data.replay_path+"/"+data.replay_hash+"-"+data.replay_filename,"/home/web_user/content/replay.replay1",false));
      }
      proms.push(fetchfs.registerFetchFS({"retroarch_web_base.cfg":null}, "/assets", "/home/web_user/retroarch/", false));
      fetchfs.mkdirp(saves_dir);
      fetchfs.mkdirp(state_dir);
      Promise.all(proms).then(function () {
        copyFile("/home/web_user/retroarch/retroarch_web_base.cfg", "/home/web_user/retroarch/userdata/retroarch.cfg");
        // TODO if movie, it would be very cool to have a screenshot of the movie's init state copied in here
        if (entryState) {
          copyFile("/home/web_user/content/entry_state",
            state_dir + "/" + content_base + ".state1.entry");
          copyFile("/home/web_user/content/entry_state",
            state_dir + "/" + content_base + ".state1");
          if(file_exists("/home/web_user/content/entry_state.png")) {
            copyFile("/home/web_user/content/entry_state.png", state_dir+"/"+content_base+".state1.png");
          } else {
            fetch(IMG_STATE_ENTRY).then((resp) => resp.arrayBuffer()).then((buf) => FS.writeFile(state_dir+"/"+content_base+".state1.png", new Uint8Array(buf)));
          }
        }
        if (movie) {
          console.log("Put movie in",state_dir + "/" + content_base + ".replay1");
          copyFile("/home/web_user/content/replay.replay1",
            state_dir + "/" + content_base + ".replay1");
          // if(file_exists("/home/web_user/content/entry_state.png")) {
          //   copyFile("/home/web_user/content/entry_state.png", state_dir+"/"+content_base+".state1.png");
          // } else {
          //   fetch(IMG_STATE_ENTRY).then((resp) => resp.arrayBuffer()).then((buf) => FS.writeFile(state_dir+"/"+content_base+".state1.png", new Uint8Array(buf)));
          // }
        } else {
          const f = FS.open(state_dir+"/"+content_base+".replay1", 'w');
          const te = new TextEncoder();
          FS.write(f, te.encode("\0"), 0, 1);
          FS.close(f);
        }
        retroReady();
      });
    });
}

function copyFile(from: string, to: string): void {
  let buf = FS.readFile(from);
  FS.writeFile(to, buf);
}

// TODO add clear button to call ui_state.clear()
function retroReady(): void {
  ui_state = new UI(
    <HTMLDivElement>document.getElementById("ui")!,
      {
        "load_state": (num: number) => load_state_slot(num),
        "load_checkpoint": (num: number) => load_state_slot(num),
        "play_replay": (num: number) => play_replay_slot(num),
        "download_file": (category: "state" | "save" | "replay", file_name: string) => {
          let path = "/home/web_user/retroarch/userdata";
          if (category == "state") {
            path += "/states";
          } else if (category == "save") {
            path += "/saves";
          } else if (category == "replay") {
            path += "/states";
          } else {
            console.error("Invalid save category", category, file_name);
          }
          const data = FS.readFile(path + "/" + file_name);
          saveAs(new Blob([data]), file_name);
        }
      }
  );

  let prev = document.getElementById("webplayer-preview")!;
  prev.classList.add("loaded");
  prev.addEventListener(
    "click",
    function () {
      let canv = <HTMLCanvasElement>document.getElementById("canvas")!;
      prev.classList.add("hidden");
      startRetroArch(canv, retro_args, function () {
        let canv = document.getElementById("canvas")!;
        setInterval(checkChangedStatesAndSaves, FS_CHECK_INTERVAL);
        canv.classList.remove("hidden");
      });
      return false;
    });
}
function nonnull(obj:any):asserts obj {
  if(obj == null) {
    throw "Must be non-null";
  }
}

function load_state_slot(n:number) {
  send_message("LOAD_STATE_SLOT "+n.toString());
}
async function play_replay_slot(n:number) {
  clear_current_replay();
  send_message("PLAY_REPLAY_SLOT "+n.toString());
  let resp = await read_response(true);
  nonnull(resp);
  const num_str = (resp.match(/PLAY_REPLAY_SLOT ([0-9]+)$/)?.[1]) ?? "0";
  if(num_str == "0") {
    return;
  }
  current_replay = {mode:ReplayMode.Playback,id:num_str,finished:false};
  find_checkpoints_inner();
}
enum BSVFlags {
  START_RECORDING    = (1 << 0),
  START_PLAYBACK     = (1 << 1),
  PLAYBACK           = (1 << 2),
  RECORDING          = (1 << 3),
  END                = (1 << 4),
  EOF_EXIT           = (1 << 5)
}

async function read_response(wait:boolean): Promise<string | null> {
  const waiting:() => Promise<string|null> = () => new Promise((resolve,_reject) => {
    let interval:number;
    const read_cb = () => {
      let resp = retroArchRecv();
      if(resp != null) {
        clearInterval(interval!);
        resolve(resp);
      }
    }
    interval = setInterval(read_cb, 100);
  });
  let outp:string|null=null;
  if(wait) {
    outp = await waiting();
  } else {
    outp = retroArchRecv();
  }
  console.log("stdout: ",outp);
  return outp;
}

async function send_message(msg:string) {
  let clearout = await read_response(false);
  while(clearout) { clearout = await read_response(false); }
  console.log("send:",msg);
  retroArchSend(msg+"\n");
}
// Called by timer from time to time
async function update_checkpoints() {
  await send_message("GET_CONFIG_PARAM active_replay");
  let resp = await read_response(true);
  nonnull(resp);
  let matches = resp.match(/GET_CONFIG_PARAM active_replay ([0-9]+) ([0-9]+)$/);
  const id = (matches?.[1]) ?? "0";
  const flags = parseInt((matches?.[2]) ?? "0",10);
  if(id == "0" || flags == 0) {
    console.log("no current replay or different replay started");
    clear_current_replay();
  } else {
    if(current_replay && current_replay.id != id) {
      clear_current_replay();
    }
    let finished = (flags & BSVFlags.END) != 0;
    let mode = (flags & BSVFlags.PLAYBACK) != 0 ? ReplayMode.Playback : (flags & BSVFlags.RECORDING ? ReplayMode.Record : ReplayMode.Inactive);
    console.log("current replay",id,mode,finished);
    current_replay = {id:id,mode:mode,finished:finished};
  }
  if(current_replay) {
    find_checkpoints_inner();
  }
}
function find_checkpoints_inner() {
  nonnull(current_replay);
  // search state files for states saved of current replay
  console.log(seen_states);
  for(let state_file in seen_states) {
    if(state_file in seen_checkpoints) { continue; }
    console.log("Check ",state_file);
    const replay = ra_util.replay_of_state(new Uint8Array(FS.readFile(state_dir+"/"+state_file)));
    console.log("Replay info",replay,"vs",current_replay);
    if(replay && replay.id == current_replay.id) {
      seen_checkpoints[state_file] = seen_states[state_file];
      ui_state.newCheckpoint(state_file, base64EncArr(seen_states[state_file]));
    }
  }
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

let current_replay:Replay | null = null;
const seen_states:Record<string,Uint8Array> = {};
const seen_saves:Record<string,null> = {};
const seen_replays:Record<string,string> = {};
let seen_checkpoints:Record<string,Uint8Array> = {};
function checkChangedStatesAndSaves() {
  const states = FS.readdir(state_dir);
  for (let state of states) {
    if(state == "." || state == "..") { continue; }
    if(state.endsWith(".png") || state.includes(".state")) {
      const png_file = state.endsWith(".png") ? state : state + ".png";
      const state_file = state.endsWith(".png") ? state.substring(0,state.length-4) : state;
      if(state_file in seen_states || state_file in seen_checkpoints) {
        continue;
      }
      // If not yet seen and both files exist
      if(!(file_exists(state_dir+"/"+png_file) && file_exists(state_dir+"/"+state_file))) {
        continue;
      }
      console.log("check state file",state);
      const replay = ra_util.replay_of_state((FS.readFile(state_dir+"/"+state_file)));
      let known_replay = false;
      if(replay) {
        for(let seen in seen_replays) {
          if(seen_replays[seen] == replay.id) {
            known_replay = true;
          }
        }
      }
      const img_data = FS.readFile(state_dir+"/"+png_file);
      const img_data_b64 = base64EncArr(img_data);
      // If this state belongs to the current replay...
      if(replay && current_replay && replay.id == current_replay.id) {
        seen_checkpoints[state_file] = img_data;
        ui_state.newCheckpoint(state_file, img_data_b64);
        // otherwise ignore it if it's a checkpoint from a non-current replay we have locally
      } else if(!replay || !known_replay) {
        seen_states[state_file] = img_data;
        // TODO something's off here, states of non current replays are getting added here
        ui_state.newState(state_file, img_data_b64);
      }
    } else if(state.includes(".replay")) {
      if(!(state in seen_replays)) {
        const replay = ra_util.replay_info(new Uint8Array(FS.readFile(state_dir+"/"+state)));
        seen_replays[state] = replay.id;
        ui_state.newReplay(state);
      }
    }
  }
  const saves = FS.readdir(saves_dir);
  for (let save of saves) {
    if(save == "." || save == "..") { continue; }
    if(!(save in seen_saves)) {
      seen_saves[save] = null;
      ui_state.newSave(save);
    }
  }
  update_checkpoints();
}
function clear_current_replay() {
  current_replay = null;
  seen_checkpoints = {};
  ui_state.clearCheckpoints();
}

function file_exists(path:string) : boolean {
  return FS.analyzePath(path).exists
}
