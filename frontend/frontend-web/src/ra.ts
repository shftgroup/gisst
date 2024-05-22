import * as fetchfs from './fetchfs';
import {UI, GISSTDBConnector, GISSTModels, ReplayMode as UIReplayMode} from 'gisst-player';
import {saveAs,base64EncArr} from './util';
import * as ra_util from 'ra-util';
import {ColdStart, StateStart, ReplayStart, ObjectLink} from './types';
import * as tus from "tus-js-client";
import {LibretroModule, loadRetroArch} from './libretro_adapter';
const FS_CHECK_INTERVAL = 1000;

const state_dir = "/home/web_user/retroarch/userdata/states";
const saves_dir = "/home/web_user/retroarch/userdata/saves";

const retro_args = ["-v"];

let RA:LibretroModule;
let ui_state:UI;
let db:GISSTDBConnector;

export function init(core:string, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[], boot_into_record:boolean) {
  db = new GISSTDBConnector(`${window.location.protocol}//${window.location.host}`);

  const content = manifest.find((o) => o.object_role=="content")!;
  const content_file = content.file_filename!;
  const content_base = content_file.substring(0, content_file.lastIndexOf("."));
  const entryState = start.type == "state";
  const movie = start.type == "replay";
  // TODO detect mobile or whatever
  const use_gamepad_overlay = true;
  if (entryState) {
    retro_args.push("-e");
    retro_args.push("1");
  }
  if (movie) {
    retro_args.push("-P");
    retro_args.push(state_dir+"/"+content_base+".replay1");
  } else if(boot_into_record) {
    retro_args.push("-R");
    retro_args.push(state_dir+"/"+content_base+".replay1");
  }
  retro_args.push("-c");
  retro_args.push("/home/web_user/retroarch/userdata/retroarch.cfg");
  const has_config = manifest.find((o) => o.object_role=="config")!;
  if(has_config) {
    retro_args.push("--appendconfig");
    retro_args.push("/home/web_user/content/retroarch.cfg");
  }
  retro_args.push("/home/web_user/content/" + content.file_source_path! + "/" + content.file_filename!);
  console.log(retro_args);

  ui_state = new UI(
    <HTMLDivElement>document.getElementById("ui")!,
      {
        "toggle_mute": () => send_message("MUTE"),
        "load_state": (num: number) => load_state_slot(num),
        "save_state": () => save_state(),
        "load_checkpoint": (num: number) => load_state_slot(num),
        "play_replay": (num: number) => play_replay_slot(num),
        "start_replay": () => record_replay(),
        "stop_and_save_replay": () => stop_replay(),
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
          const data = RA.FS.readFile(path + "/" + file_name);
          saveAs(new Blob([data]), file_name);
        },
        "upload_file": (category: "state" | "save" | "replay", file_name: string, metadata:GISSTModels.Metadata ) => {
          return new Promise((resolve, reject) => {
            let path = "/home/web_user/retroarch/userdata";
            if (category === "state") {
              path += "/states";
            } else if (category === "save") {
              path += "/saves";
            } else if (category === "replay") {
              path += "/states";
            } else {
              console.error("Invalid save category", category, file_name);
              reject("Invalid save category:" + category + ":" + file_name)
            }
            const data = RA.FS.readFile(path + "/" + file_name);

            db.uploadFile(new File([data], file_name),
                (error:Error) => { reject(console.error("ran error callback", error.message))},
                (_percentage: number) => {},
                (upload: tus.Upload) => {
                  const url_parts = upload.url!.split('/');
                  const uuid_string = url_parts[url_parts.length - 1];
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
                  }
                })
                .catch(() => reject("File upload from RA failed."));

          })
        }
      },
      false,
      JSON.parse(document.getElementById("config")!.textContent!)
  );
  ui_state.setReplayMode(movie ? UIReplayMode.Playback : (boot_into_record ? UIReplayMode.Record : UIReplayMode.Inactive));

  loadRetroArch("", core,
    function (module:LibretroModule) {
      RA = module;
      fetchfs.mkdirp(RA,"/home/web_user/content");
      fetchfs.mkdirp(RA, saves_dir);
      fetchfs.mkdirp(RA, state_dir);

      const proms = [];

      proms.push(fetchfs.fetchZip(RA,"/assets/frontend/bundle.zip","/home/web_user/retroarch/"));

      for(const file of manifest) {
        const source_path = "/home/web_user/content/" + file.file_source_path;
        fetchfs.mkdirp(RA, source_path);
        const file_prom = fetchfs.fetchFile(
            RA,
            "/storage/" + file.file_dest_path + "/" + file.file_hash + "-" + file.file_filename,
            source_path + "/" + file.file_filename);
        proms.push(file_prom);
      }
      let entryScreenshot:Promise<GISSTModels.DBRecord> | null = null;
      if (entryState) {
        // Cast: This one is definitely a statestart because the type is state
        const data = (start as StateStart).data;
        console.log(data, "/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/entry_state");
        if(!data.screenshot_id) {
          console.error("No screenshot for entry state");
          entryScreenshot = Promise.resolve({screenshot_id:"", screenshot_data:""});
        } else {
          entryScreenshot = db.getRecordById("screenshot", data.screenshot_id);
          proms.push(entryScreenshot);
        }
        proms.push(fetchfs.fetchFile(RA,"/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/entry_state"));
      }
      if (movie) {
        // Cast: This one is definitely a replaystart because the type is state
        const data = (start as ReplayStart).data;
        console.log(data, "/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/replay.replay1");
        proms.push(fetchfs.fetchFile(RA, "/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/replay.replay1"));
      }
      proms.push(fetchfs.fetchFile(RA, "/assets/retroarch_web_base.cfg", "/home/web_user/retroarch/userdata/retroarch.cfg"));
      Promise.all(proms).then(function () {
        if (use_gamepad_overlay) {
          // gameboy, gba, nes, snes, retropad
          // gambatte, vba_next, fceumm, snes9x
          const overlays = {
            "gambatte": "gameboy",
            "vba_next": "gba",
            "fceumm": "nes",
            "snes9x": "snes"
          };
          let overlay = "retropad";
          if (core in overlays) {
            overlay = overlays[core as keyof typeof overlays];
          }
          const lines = "\ninput_overlay_enable = \"true\"\ninput_overlay = \"/home/web_user/retroarch/bundle/overlays/gamepads/"+overlay+"/"+overlay+".cfg\"\ninput_overlay_enable_autopreferred = \"true\"";
          const enc = new TextEncoder();
          const lines_enc = enc.encode(lines);
          const cfg = module.FS.open("/home/web_user/retroarch/userdata/retroarch.cfg", "a");
          module.FS.write(cfg,lines_enc,0,lines_enc.length);
          module.FS.close(cfg);
        }

        // TODO if movie, it would be very cool to have a screenshot of the movie's init state copied in here
        if (entryState) {
          copyFile(RA, "/home/web_user/content/entry_state",
            state_dir + "/" + content_base + ".state1.entry");
          copyFile(RA, "/home/web_user/content/entry_state",
            state_dir + "/" + content_base + ".state1");
          if(file_exists(RA, "/home/web_user/content/entry_state.png")) {
            copyFile(RA, "/home/web_user/content/entry_state.png", state_dir+"/"+content_base+".state1.png");
          } else {
            const data = (start as StateStart).data;
            // entryScreenshot is already settled from the all() above
            entryScreenshot!.then((screenshot) => {
              seen_states[content_base+".state1"] = (screenshot as GISSTModels.Screenshot).screenshot_data;
              ui_state.newState(content_base+".state1", (screenshot as GISSTModels.Screenshot).screenshot_data, data);
            });
          }
        }
        if (movie) {
          console.log("Put movie in",state_dir + "/" + content_base + ".replay1");
          copyFile(RA, "/home/web_user/content/replay.replay1", state_dir + "/" + content_base + ".replay1");
          const data = (start as ReplayStart).data;
          // TODO it's ugly to read this in again right after downloading it but whatever
          const replayUUID = ra_util.replay_info(new Uint8Array(RA.FS.readFile("/home/web_user/content/replay.replay1"))).id;
          seen_replays[content_base+".replay1"] = replayUUID;
          ui_state.newReplay(content_base+".replay1", data);
        } else if(boot_into_record) {
          const f = RA.FS.open(state_dir+"/"+content_base+".replay1", 'w');
          const te = new TextEncoder();
          RA.FS.write(f, te.encode("\0"), 0, 1);
          RA.FS.close(f);
        }
        retroReady();
      });
    });
}

function copyFile(module:LibretroModule, from: string, to: string): void {
  const buf = module.FS.readFile(from);
  module.FS.writeFile(to, buf);
}

// TODO add clear button to call ui_state.clear()
function retroReady(): void {
  const prev = document.getElementById("webplayer-preview")!;
  prev.classList.add("loaded");
  prev.addEventListener(
    "click",
    function () {
      const canv = <HTMLCanvasElement>document.getElementById("canvas")!;
      prev.classList.add("hidden");
      RA.startRetroArch(canv, retro_args, function () {
        setInterval(checkChangedStatesAndSaves, FS_CHECK_INTERVAL);
        canv.classList.remove("hidden");
      });
      return false;
    });
}
function nonnull(obj:unknown):asserts obj {
  if(obj == null) {
    throw "Must be non-null";
  }
}

function load_state_slot(n:number) {
  send_message("LOAD_STATE_SLOT "+n.toString());
}

function save_state() {
  send_message("SAVE_STATE");
}

function record_replay() {
  send_message("RECORD_REPLAY");
}

function stop_replay() {
  send_message("HALT_REPLAY");
}

async function play_replay_slot(n:number) {
  clear_current_replay();
  send_message("PLAY_REPLAY_SLOT "+n.toString());
  const resp = await read_response(true);
  nonnull(resp);
  const num_str = (resp.match(/PLAY_REPLAY_SLOT ([0-9]+)$/)?.[1]) ?? "0";
  if(num_str == "0") {
    return;
  }
  current_replay = {mode:ReplayMode.Playback,id:num_str,finished:false};
  ui_state.setReplayMode(ReplayMode.Playback);
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

async function read_response(wait:boolean): Promise<string | undefined> {
  const waiting:() => Promise<string|undefined> = () => new Promise((resolve) => {
    /* eslint-disable prefer-const */
    let interval:ReturnType<typeof setInterval>;
    const read_cb = () => {
      const resp = RA.retroArchRecv();
      if(resp != undefined) {
        clearInterval(interval!);
        resolve(resp);
      }
    }
    interval = setInterval(read_cb, 100);
  });
  let outp:string|undefined=undefined;
  if(wait) {
    outp = await waiting();
  } else {
    outp = RA.retroArchRecv();
  }
  // console.log("stdout: ",outp);
  return outp;
}

async function send_message(msg:string) {
  let clearout = await read_response(false);
  while(clearout) { clearout = await read_response(false); }
  // console.log("send:",msg);
  RA.retroArchSend(msg+"\n");
}
// Called by timer from time to time
async function update_checkpoints() {
  await send_message("GET_CONFIG_PARAM active_replay");
  const resp = await read_response(true);
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
  if(current_replay) {
    find_checkpoints_inner();
  }
}
function find_checkpoints_inner() {
  nonnull(current_replay);
  // search state files for states saved of current replay
  // console.log("seen:",seen_states);
  for(const state_file in seen_states) {
    if(state_file in seen_checkpoints) { continue; }
    // console.log("Check ",state_file);
    const replay = ra_util.replay_of_state(new Uint8Array(RA.FS.readFile(state_dir+"/"+state_file)));
    // console.log("Replay info",replay,"vs",current_replay);
    if(replay && replay.id == current_replay.id) {
      seen_checkpoints[state_file] = seen_states[state_file];
      ui_state.newCheckpoint(state_file, seen_states[state_file]);
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
// const seen_saves:Record<string,null> = {};
const seen_replays:Record<string,string> = {};
let seen_checkpoints:Record<string,string> = {};
function checkChangedStatesAndSaves() {
  const states = RA.FS.readdir(state_dir);
  for (const state of states) {
    if(state == "." || state == "..") { continue; }
    if(state.endsWith(".png") || state.includes(".state")) {
      // console.log("check state file",state);
      const png_file = state.endsWith(".png") ? state : state + ".png";
      const state_file = state.endsWith(".png") ? state.substring(0,state.length-4) : state;
      if(state_file in seen_states || state_file in seen_checkpoints) {
        continue;
      }
      // If not yet seen and both files exist
      if(!(file_exists(RA,state_dir+"/"+png_file) && file_exists(RA,state_dir+"/"+state_file))) {
        continue;
      }
      // console.log("check state file",state);
      const replay = ra_util.replay_of_state((RA.FS.readFile(state_dir+"/"+state_file)));
      let known_replay = false;
      if(replay) {
        for(const seen in seen_replays) {
          if(seen_replays[seen] == replay.id) {
            known_replay = true;
          }
        }
      }
      const img_data = RA.FS.readFile(state_dir+"/"+png_file);
      const img_data_b64 = base64EncArr(img_data);
      // If this state belongs to the current replay...
      if(replay && current_replay && replay.id == current_replay.id) {
        seen_checkpoints[state_file] = img_data_b64;
        ui_state.newCheckpoint(state_file, img_data_b64);
        // otherwise ignore it if it's a checkpoint from a non-current replay we have locally
      } else if(!replay || !known_replay) {
        seen_states[state_file] = img_data_b64;
        ui_state.newState(state_file, img_data_b64);
      }
    } else if(state.includes(".replay")) {
      if(!(state in seen_replays)) {
        const replay = ra_util.replay_info(new Uint8Array(RA.FS.readFile(state_dir+"/"+state)));
        seen_replays[state] = replay.id;
        ui_state.newReplay(state);
      }
    }
  }
  // const saves = RA.FS.readdir(saves_dir);
  // for (const save of saves) {
  //   if(save == "." || save == "..") { continue; }
  //   if(!(save in seen_saves)) {
  //     seen_saves[save] = null;
  //     ui_state.newSave(save);
  //   }
  // }
  update_checkpoints();
}
function clear_current_replay() {
  current_replay = null;
  seen_checkpoints = {};
  ui_state.clearCheckpoints();
  ui_state.setReplayMode(UIReplayMode.Inactive);
}

function file_exists(RA:LibretroModule, path:string) : boolean {
  return RA.FS.analyzePath(path).exists
}
