import {UI, GISSTDBConnector, GISSTModels, ReplayMode as UIReplayMode, ZoomLevel} from 'gisst-player';
import {saveAs,base64EncArr} from './util';
import * as ra_util from 'ra-util';
import type {StateStart, ReplayStart, RetroarchCore, EmuControls} from 'frontend-embed/types';

// Debugging aids
declare global {
  interface Window { RA: RetroarchCore, UI: UI; DB: GISSTDBConnector; }
}

const FS_CHECK_INTERVAL = 1000;
// one auto state per 5 minutes feels reasonable
const AUTO_STATE_INTERVAL = 5*60*1000;

let state_dir = "";
let saves_dir = "";

let RA:RetroarchCore;
let ui_state:UI;
let db:GISSTDBConnector;

let content_base = "";

let base_width:number = 480;
let base_height:number = 360;
let zoom_fit = false;

function zoom_to_fit() {
  const canv = <HTMLCanvasElement>ui_state.emulator_div.querySelector("canvas")!;
  const content = <HTMLDivElement>document.querySelector(".gisst-internal-content-body")!;
  const n = Math.min(content.clientWidth/base_width,content.clientHeight/base_height);
  canv.style.width = `${base_width*n}px`;
  canv.style.height = `${base_height*n}px`;
}

export async function init(ui:UI, embed:EmuControls) {
  ui_state = ui;
  RA = embed.inner as RetroarchCore;
  content_base = RA.content_name;
  state_dir = RA.state_dir;
  saves_dir = RA.saves_dir;
  const gisst_root = embed.gisst_root;
  db = new GISSTDBConnector(gisst_root);
  ui_state.setControl(
    {
      "evt_to_html": (evt:unknown) => {
        const elt = document.createElement("span");
        elt.innerText=evt as string;
        return elt;
      },
      "activate_save": (savefile:string) => activate_save(savefile),
      "create_save": () => create_save(),
      "toggle_mute": () => embed.toggle_mute(),
      "set_zoom": (level:ZoomLevel) => {
        const canv = <HTMLCanvasElement>ui_state.emulator_div.querySelector("canvas")!;
        zoom_fit = level == ZoomLevel.Fit;
        switch(level) {
          case ZoomLevel.X05:
            canv.style.width = `${base_width*0.5}px`;
            canv.style.height = `${base_height*0.5}px`;
            break;
          case ZoomLevel.X1:
            canv.style.width = `${base_width}px`;
            canv.style.height = `${base_height}px`;
            break;
          case ZoomLevel.X2:
            canv.style.width = `${base_width*2}px`;
            canv.style.height = `${base_height*2}px`;
            break;
          case ZoomLevel.Fit: {
            zoom_to_fit();
            break;
          }
        }
      },
      "enter_fullscreen": () => {
        zoom_fit = false;
        RA.send_message("FULLSCREEN_TOGGLE");
      },
      "load_state": (num: number) => load_state_slot(num),
      "save_state": () => save_state(),
      "play_replay": (num: number) => play_replay_slot(num),
      "start_replay": () => record_replay(),
      "stop_and_save_replay": () => stop_replay(),
      "checkpoints_of":(replay_slot:number) => {
        const replay_file = state_dir+"/"+content_base+".replay"+replay_slot;
        const replay = ra_util.replay_info(new Uint8Array(RA.module.FS.readFile(replay_file))).id;
        const checkpoints = [];
        for(const state of Object.keys(seen_states)) {
          const state_replay = ra_util.replay_of_state((RA.module.FS.readFile(state_dir+"/"+state)))?.id;
          if(state_replay == replay) {
            checkpoints.push(state);
          }
        }
        return checkpoints;
      },
      "download_file": (category: "state" | "save" | "replay", file_name: string) => {
        let path;
        if (category == "state") {
          path = state_dir;
        } else if (category == "save") {
          path = saves_dir;
        } else if (category == "replay") {
          path = state_dir;
        } else {
          console.error("Invalid save category", category, file_name);
          return;
        }
        const data = RA.module.FS.readFile(path + "/" + file_name);
        saveAs(new Blob([new Uint8Array(data)]), file_name);
      },
      "upload_file": (category: "state" | "save" | "replay", file_name: string, metadata:GISSTModels.Metadata ) => {
        return new Promise((resolve, reject) => {
          let path;
          if (category === "state") {
            path = state_dir;
          } else if (category === "save") {
            path = saves_dir;
          } else if (category === "replay") {
            path = state_dir;
          } else {
            console.error("Invalid save category", category, file_name);
            reject("Invalid save category:" + category + ":" + file_name);
            return;
          }
          const data = RA.module.FS.readFile(path + "/" + file_name);

          db.uploadFile(new File([new Uint8Array(data)], file_name), metadata.record.file_id,
            (error:Error) => { reject(console.error("ran error callback", error.message))},
            (_percentage: number) => {},
            (uuid_string: string) => {
              metadata.record.file_id = uuid_string;
              if (category === "state"){
                db.uploadRecord({screenshot_data: metadata.screenshot}, "screenshot")
                  .then((screenshot:GISSTModels.DBRecord) => {
                    (metadata.record as GISSTModels.State).screenshot_id = (screenshot as GISSTModels.Screenshot).screenshot_id;
                    db.uploadRecord(metadata.record, category)
                      .then((state:GISSTModels.DBRecord) => {
                        (metadata.record as GISSTModels.State).state_id = (state as GISSTModels.State).state_id;
                        resolve(metadata)
                      })
                      .catch((e) => {console.error(e); reject(`${category} upload from RA failed.`);})
                  })
                  .catch((e) => {console.error(e); reject("Screenshot upload from RA failed.");})
              } else if (category === "replay") {
                db.uploadRecord(metadata.record, category)
                  .then((replay:GISSTModels.DBRecord) => {
                    (metadata.record as GISSTModels.Replay).replay_id = (replay as GISSTModels.Replay).replay_id;
                    resolve(metadata)
                  })
                  .catch((e) => {console.error(e); reject(`${category} upload from RA failed.`);})
                // upload all associated states too
              } else if (category === "save") {
                db.uploadRecord(metadata.record, category)
                  .then((save:GISSTModels.DBRecord) => {
                    (metadata.record as GISSTModels.Save).save_id = (save as GISSTModels.Save).save_id;
                    resolve(metadata)
                  })
                  .catch((e) => {console.error(e); reject(`${category} upload from RA failed.`);})
                // upload all associated states too
              }
            })
            .catch(() => reject("File upload from RA failed."));

        })
      }
    },
  );
  let entryScreenshot:GISSTModels.DBRecord;
  if (embed.start.type == "state") {
    const data = (embed.start as StateStart).data;
    if(!data.screenshot_id) {
      console.error("No screenshot for entry state");
      entryScreenshot = {screenshot_id:"", screenshot_data:""};
    } else {
      entryScreenshot = await db.getRecordById("screenshot", data.screenshot_id);
    }
    seen_states[content_base+".state1"] = (entryScreenshot as GISSTModels.Screenshot).screenshot_data;
    ui_state.newState(content_base+".state1", (entryScreenshot as GISSTModels.Screenshot).screenshot_data, "init", data as GISSTModels.StateFileLink);
  }
  let replay:Uint8Array|null = null;
  const movie = embed.start.type == "replay";
  if (movie) {
    const data = (embed.start as ReplayStart).data;
    replay = data.replay!;
    const replayUUID = ra_util.replay_info(new Uint8Array(replay)).id;
    seen_replays[content_base+".replay1"] = replayUUID;
    ui_state.newReplay(content_base+".replay1", "init", data as GISSTModels.ReplayFileLink);
  }
  ui_state.setReplayMode(movie ? UIReplayMode.Playback : (embed.embed_options.record_from_start ? UIReplayMode.Record : UIReplayMode.Inactive));
  window.RA = RA;
  window.UI = ui_state;
  window.DB = db;
  const canv = <HTMLCanvasElement>ui_state.emulator_div.querySelector("canvas")!;
  {
    const container = canv.parentElement!;
    const ro = new ResizeObserver((_entries, _observer) => {
      if (zoom_fit) {
        zoom_to_fit();
      }
    });
    ro.observe(container);
  }
  setInterval(checkChangedStatesAndSaves, FS_CHECK_INTERVAL);
  setInterval(autosave_state, AUTO_STATE_INTERVAL);
  base_width = parseInt(canv.style.width.slice(0,canv.style.width.length-2)) || 480;
  canv.style.width = `${base_width}px`;
  canv.width = base_width;
  base_height = parseInt(canv.style.height.slice(0,canv.style.height.length-2)) || 360;
  canv.style.height = `${base_height}px`;
  canv.height = base_height;
  console.log("Base dimensions:",base_width,base_height);
}

// TODO add clear button to call ui_state.clear()
function autosave_state() {
  save_state();
}
function nonnull(obj:unknown):asserts obj {
  if(obj == null) {
    throw "Must be non-null";
  }
}

function load_state_slot(n:number) {
  RA.send_message("LOAD_STATE_SLOT "+n.toString());
}

function save_state() {
  RA.send_message("SAVE_STATE");
}

function record_replay() {
  RA.send_message("RECORD_REPLAY");
}

function stop_replay() {
  RA.send_message("HALT_REPLAY");
}

function copyFile(from:string, to:string) {
  const contents = RA.module.FS.readFile(from, {encoding:'binary'});
  try {
    RA.module.FS.unlink(to);
  } catch(_e) {
    console.log("Couldn't unlink file; is it new?",to);
  }
  RA.module.FS.writeFile(to, contents);
}

async function activate_save(savefile:string) {
  const srm = `${saves_dir}/${content_base}.srm`;
  const from = `${saves_dir}/${savefile}`;
  await create_save();
  copyFile(from, srm);
  RA.send_message("LOAD_FILES");
}

let save_counter = 0;
async function create_save() {
  RA.send_message("SAVE_FILES");
  await RA.read_response(true);
  const srm = `${saves_dir}/${content_base}.srm`;
  const to = `${saves_dir}/autosave_${save_counter}.srm`;
  save_counter++;
  copyFile(srm, to);
}

async function play_replay_slot(n:number) {
  clear_current_replay();
  RA.send_message("PLAY_REPLAY_SLOT "+n.toString());
  const resp = await RA.read_response(true);
  nonnull(resp);
  const num_str = (resp.match(/PLAY_REPLAY_SLOT ([0-9]+)$/)?.[1]) ?? "0";
  if(num_str == "0") {
    return;
  }
  current_replay = {mode:ReplayMode.Playback,id:num_str,finished:false};
  ui_state.setReplayMode(UIReplayMode.Playback);
}
enum BSVFlags {
  START_RECORDING    = (1 << 0),
  START_PLAYBACK     = (1 << 1),
  PLAYBACK           = (1 << 2),
  RECORDING          = (1 << 3),
  END                = (1 << 4),
  EOF_EXIT           = (1 << 5)
}

// Called by timer from time to time
async function update_replay_state() {
  await RA.send_message("GET_CONFIG_PARAM active_replay");
  const resp = await RA.read_response(true);
  nonnull(resp);
  const matches = resp.match(/GET_CONFIG_PARAM active_replay ([0-9]+) ([0-9]+)$/);
  const id = (matches?.[1]) ?? "0";
  const flags = parseInt((matches?.[2]) ?? "0",10);
  if(id == "0" || flags == 0) {
    // console.log("no current replay or different replay started");
    clear_current_replay();
  } else {
    if(current_replay && current_replay.id != id) {
      clear_current_replay();
    }
    const finished = (flags & BSVFlags.END) != 0;
    const mode = (flags & BSVFlags.PLAYBACK) != 0 ? ReplayMode.Playback : (flags & BSVFlags.RECORDING ? ReplayMode.Record : ReplayMode.Inactive);
    // console.log("current replay",id,mode,finished);
    current_replay = {id:id,mode:mode,finished:finished};
    if(finished) {
      ui_state.setReplayMode(UIReplayMode.Finished);
    } else {
      switch (mode) {
        case ReplayMode.Inactive:
        ui_state.setReplayMode(UIReplayMode.Inactive);
        break;
        case ReplayMode.Playback:
        ui_state.setReplayMode(UIReplayMode.Playback);
        break;
        case ReplayMode.Record:
        ui_state.setReplayMode(UIReplayMode.Record);
        break;
      }
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
const seen_states:Record<string,string> = {};
const seen_saves:string[] = [];
const seen_replays:Record<string,string> = {};
function checkChangedStatesAndSaves() {
  const states = RA.module.FS.readdir(state_dir);
  for (const state of states) {
    if(state == "." || state == "..") { continue; }
    if(state.endsWith(".png") || state.includes(".state")) {
      // console.log("check state file",state);
      const png_file = state.endsWith(".png") ? state : state + ".png";
      const state_file = state.endsWith(".png") ? state.substring(0,state.length-4) : state;
      if(state_file in seen_states) {
        continue;
      }
      // If not yet seen and both files exist
      if(!(file_exists(RA,state_dir+"/"+png_file) && file_exists(RA,state_dir+"/"+state_file))) {
        continue;
      }
      // console.log("check state file",state);
      const replay = ra_util.replay_of_state((RA.module.FS.readFile(state_dir+"/"+state_file)));
      const img_data = RA.module.FS.readFile(state_dir+"/"+png_file);
      const img_data_b64 = base64EncArr(img_data);
      seen_states[state_file] = img_data_b64;
      ui_state.newState(state_file, img_data_b64, replay?.id);
    } else if(state.includes(".replay")) {
      if(!(state in seen_replays)) {
        const replay = ra_util.replay_info(new Uint8Array(RA.module.FS.readFile(state_dir+"/"+state)));
        seen_replays[state] = replay.id;
        ui_state.newReplay(state, replay.id);
      }
    }
  }
  update_replay_state();
  const saves = RA.module.FS.readdir(saves_dir);
  for (const save of saves) {
    if(save.endsWith(".srm") && save != `${content_base}.srm`) {
      if(!(seen_saves.includes(save))) {
        seen_saves.push(save);
        ui_state.newSave(save);
      }
    }
  }
}
function clear_current_replay() {
  current_replay = null;
  ui_state.setReplayMode(UIReplayMode.Inactive);
}

function file_exists(RA:RetroarchCore, path:string) : boolean {
  return RA.module.FS.analyzePath(path).exists
}

