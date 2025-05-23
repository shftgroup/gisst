import {UI, GISSTDBConnector, GISSTModels, ReplayMode as UIReplayMode} from 'gisst-player';
import {saveAs,base64EncArr} from './util';
import * as ra_util from 'ra-util';
import {ColdStart, StateStart, ReplayStart, ObjectLink, EmbedOptions, ControllerOverlayMode} from './types.d';
import {LibretroModule, loadRetroArch} from './libretro_adapter';

const FS_CHECK_INTERVAL = 1000;
// one auto state per 5 minutes feels reasonable
const AUTO_STATE_INTERVAL = 5*60*1000;

const state_dir = "/mem/states";
const saves_dir = "/mem/saves";

const retro_args = ["-v"];

let RA:LibretroModule;
let ui_state:UI<string>;
let db:GISSTDBConnector;

let content_base:string;

export async function init(gisst_root:string, core:string, start:ColdStart | StateStart | ReplayStart, saves:GISSTModels.SaveFileLink[], manifest:ObjectLink[], boot_into_record:boolean, embed_options:EmbedOptions) {
  db = new GISSTDBConnector(gisst_root);

  const content = manifest.find((o) => o.object_role=="content" && o.object_role_index == 0)!;
  const content_file = content.file_filename!;
  content_base = content_file.substring(0, content_file.lastIndexOf("."));
  const entryState = start.type == "state";
  const movie = start.type == "replay";
  const source_path = content.file_source_path!.replace(content.file_filename!, "");
  const use_gamepad_overlay = embed_options.controls == ControllerOverlayMode.On || ((embed_options.controls??ControllerOverlayMode.Auto) == ControllerOverlayMode.Auto && mobileAndTabletCheck());
  let entryScreenshot:GISSTModels.DBRecord;
  let state_data:Uint8Array|null = null;
  if (entryState) {
    retro_args.push("-e");
    retro_args.push("1");
    const data = (start as StateStart).data;
    state_data = new Uint8Array(await (await fetch(gisst_root+"/storage/"+data.file_dest_path)).arrayBuffer());
    console.log(data, gisst_root+"/storage/"+data.file_dest_path,"/fetch/content/entry_state");
    if(!data.screenshot_id) {
      console.error("No screenshot for entry state");
      entryScreenshot = {screenshot_id:"", screenshot_data:""};
    } else {
      entryScreenshot = await db.getRecordById("screenshot", data.screenshot_id);
    }
  }
  let replay:Uint8Array|null = null;
  if (movie) {
    retro_args.push("-P");
    retro_args.push(state_dir+"/"+content_base+".replay1");
    const data = (start as ReplayStart).data;
    replay = new Uint8Array(await ((await fetch(gisst_root+"/storage/"+data.file_dest_path)).arrayBuffer()));
  } else if(boot_into_record) {
    retro_args.push("-R");
    retro_args.push(state_dir+"/"+content_base+".replay1");
  }
  const save_data:[GISSTModels.SaveFileLink,Uint8Array][] = [];
  for (const save of saves) {
    save_data.push([save, new Uint8Array(await ((await fetch(gisst_root+"/storage/"+save.file_dest_path)).arrayBuffer()))]);
  }
  retro_args.push("-c");
  retro_args.push("/mem/retroarch.cfg");
  const has_config = manifest.find((o) => o.object_role=="config")!;
  if(has_config) {
    retro_args.push("--appendconfig");
    retro_args.push("/fetch/content/retroarch.cfg");
  }
  retro_args.push("/fetch/content/" + source_path + "/" + content.file_filename!);
  console.log(retro_args);
  let ra_cfg_text:string = await ((await fetch(gisst_root+"/assets/retroarch_web_base.cfg")).text());

  ui_state = new UI(
    <HTMLDivElement>document.getElementById("ui")!,
    {
      "evt_to_html": (evt:string) => {
        const elt = document.createElement("span");
        elt.innerText=evt;
        return elt;
      },
      "activate_save": (savefile) => activate_save(savefile),
      "create_save": () => create_save(),
      "toggle_mute": () => send_message("MUTE"),
      "load_state": (num: number) => load_state_slot(num),
      "save_state": () => save_state(),
      "play_replay": (num: number) => play_replay_slot(num),
      "start_replay": () => record_replay(),
      "stop_and_save_replay": () => stop_replay(),
      "checkpoints_of":(replay_slot:number) => {
        const replay_file = state_dir+"/"+content_base+".replay"+replay_slot;
        const replay = ra_util.replay_info(new Uint8Array(RA.FS.readFile(replay_file))).id;
        const checkpoints = [];
        for(const state of Object.keys(seen_states)) {
          const state_replay = ra_util.replay_of_state((RA.FS.readFile(state_dir+"/"+state)))?.id;
          if(state_replay == replay) {
            checkpoints.push(state);
          }
        }
        return checkpoints;
      },
      "download_file": (category: "state" | "save" | "replay", file_name: string) => {
        let path = "/mem";
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
          let path = "/mem";
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

          db.uploadFile(new File([data], file_name), metadata.record.file_id,
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
    false,
    JSON.parse(document.getElementById("config")!.textContent!)
  );
  ui_state.setReplayMode(movie ? UIReplayMode.Playback : (boot_into_record ? UIReplayMode.Record : UIReplayMode.Inactive));
  loadRetroArch(gisst_root, core, {'OPFS_MOUNT':'/home/web_user/retroarch', 'FETCH_MANIFEST':'/mem/fetch.txt', 'FETCH_BASE_DIR':'/fetchfs/'},
    true,
    async function (module:LibretroModule) {
      const enc = new TextEncoder();
      RA = module;
      RA.FS.createPath("/", "fetch/content", true, true);
      RA.FS.createPath("/", state_dir, true, true);
      RA.FS.createPath("/", saves_dir, true, true);
      let fetch_manifest = `${gisst_root}/storage/\n`;
      /* TODO many of these awaits could be instead done simultaneously with Promise.all() */

      for(const file of manifest) {
        let sep = file.file_source_path.startsWith("/") ? "" : "/";
        let download_source_path_full = "/fetch/content" + sep + file.file_source_path;
        let download_source_path = download_source_path_full;
        const last_index = download_source_path.lastIndexOf(file.file_filename!);
        if(last_index >= 0) {
          download_source_path = download_source_path_full.substring(0, last_index);
        } else {
          sep = download_source_path_full.endsWith("/") ? "" : "/";
          download_source_path_full += sep + file.file_filename!;
        }
        const content_url = gisst_root+"/storage/"+file.file_dest_path;
        const resp = await fetch(content_url, {method:"HEAD"});
        let sz = 0;
        if (resp.status == 200) {
          sz = parseInt(resp.headers.get("Content-Length") ?? "0",10);
        }
        await resp.text();
        if (sz > 0 && sz <= 16*1024*1024) {
          const data = await (await fetch(content_url)).arrayBuffer();
          module.FS.createPath("/",download_source_path, true, true);
          module.FS.createDataFile(download_source_path, file.file_filename!, new Uint8Array(data), true, true, true);
        } else {
          const content_url_encoded = encodeURI(file.file_dest_path);
          fetch_manifest += `${content_url_encoded} ${download_source_path_full}\n`;
        }
      }
      console.log("Place fetch manifest",fetch_manifest);
      module.FS.createDataFile("/mem", "fetch.txt", enc.encode(fetch_manifest), true, true, true);
      if (entryState) {
        module.FS.createDataFile(state_dir, content_base + ".state1.entry", state_data, true, true, true);
        module.FS.createDataFile(state_dir, content_base + ".state1", state_data, true, true, true);
      }
      if (movie) {
        const data = (start as ReplayStart).data;
        console.log(data, "/storage/"+data.file_dest_path,state_dir + "/" + content_base + ".replay1");
        module.FS.createPath("/", state_dir, true, true);
        RA.FS.createDataFile(state_dir, content_base + ".replay1", replay, true, true, false);
      }
      for (const [data, savefile] of save_data) {
        console.log(data, "/storage/"+data.file_dest_path,saves_dir + "/" + data.save_id + ".srm");
        module.FS.createPath("/", saves_dir, true, true);
        RA.FS.createDataFile(saves_dir, data.save_id + ".srm", savefile, true, true, false);
      }
      if (save_data.length > 0) {
        RA.FS.createDataFile(saves_dir, content_base + ".srm", save_data[0][1], true, true, false);
      }
      if (use_gamepad_overlay) {
        // gameboy, gba, nes, snes, retropad
        // gambatte, vba_next, fceumm, snes9x
        const overlays = {
          "gambatte": "gameboy",
          "vba_next": "gba",
          "fceumm": "nes",
          "snes9x": "snes",
        };
        let overlay = "retropad";
        if (core in overlays) {
          overlay = overlays[core as keyof typeof overlays];
        }
        ra_cfg_text += "\ninput_overlay_enable = \"true\"\ninput_overlay = \"/home/web_user/retroarch/overlays/gamepads/"+overlay+"/"+overlay+".cfg\"\ninput_overlay_enable_autopreferred = \"true\"";
      }
      const lines_enc = enc.encode(ra_cfg_text);
      module.FS.createDataFile("/mem", "retroarch.cfg", lines_enc, true, true, true);
      if (entryState) {
        const data = (start as StateStart).data;
        seen_states[content_base+".state1"] = (entryScreenshot as GISSTModels.Screenshot).screenshot_data;
        ui_state.newState(content_base+".state1", (entryScreenshot as GISSTModels.Screenshot).screenshot_data, "init", data);
      }
      if (replay) {
        // TODO if movie, it would be very cool to have a screenshot of the movie's init state copied in here
        const data = (start as ReplayStart).data;
        const replayUUID = ra_util.replay_info(new Uint8Array(replay)).id;
        seen_replays[content_base+".replay1"] = replayUUID;
        ui_state.newReplay(content_base+".replay1", "init", data);
      }
      for (const save of saves) {
        seen_saves.push(save.save_id+".srm");
        ui_state.newSave(save.save_id+".srm", "init", save);
      }
      retroReady();
    });
}

declare global {
  interface Window { RA: LibretroModule; UI: UI<string>; DB: GISSTDBConnector; }
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
        setInterval(autosave_state, AUTO_STATE_INTERVAL);
        canv.classList.remove("hidden");
        window.RA = RA;
        window.UI = ui_state;
        window.DB = db;
      });
      return false;
    });
}
function autosave_state() {
  save_state();
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

function copyFile(from:string, to:string) {
  const contents = RA.FS.readFile(from, {encoding:'binary'});
  try {
    RA.FS.unlink(to);
  } catch(_e) {
    console.log("Couldn't unlink file; is it new?",to);
  }
  RA.FS.writeFile(to, contents);
}

async function activate_save(savefile:string) {
  const srm = `${saves_dir}/${content_base}.srm`;
  const from = `${saves_dir}/${savefile}`;
  await create_save();
  copyFile(from, srm);
  send_message("LOAD_FILES");
}

let save_counter = 0;
async function create_save() {
  send_message("SAVE_FILES");
  await read_response(true);
  const srm = `${saves_dir}/${content_base}.srm`;
  const to = `${saves_dir}/autosave_${save_counter}.srm`;
  save_counter++;
  copyFile(srm, to);
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
async function update_replay_state() {
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
  const states = RA.FS.readdir(state_dir);
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
      const replay = ra_util.replay_of_state((RA.FS.readFile(state_dir+"/"+state_file)));
      const img_data = RA.FS.readFile(state_dir+"/"+png_file);
      const img_data_b64 = base64EncArr(img_data);
      seen_states[state_file] = img_data_b64;
      ui_state.newState(state_file, img_data_b64, replay?.id);
    } else if(state.includes(".replay")) {
      if(!(state in seen_replays)) {
        const replay = ra_util.replay_info(new Uint8Array(RA.FS.readFile(state_dir+"/"+state)));
        seen_replays[state] = replay.id;
        ui_state.newReplay(state, replay.id);
      }
    }
  }
  update_replay_state();
  const saves = RA.FS.readdir(saves_dir);
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

function file_exists(RA:LibretroModule, path:string) : boolean {
  return RA.FS.analyzePath(path).exists
}

// TYVM https://stackoverflow.com/a/11381730
function mobileAndTabletCheck() {
  let check = false;
  (function(a){if(/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino|android|ipad|playbook|silk/i.test(a)||/1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw-(n|u)|c55\/|capi|ccwa|cdm-|cell|chtm|cldc|cmd-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc-s|devi|dica|dmob|do(c|p)o|ds(12|-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(-|_)|g1 u|g560|gene|gf-5|g-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd-(m|p|t)|hei-|hi(pt|ta)|hp( i|ip)|hs-c|ht(c(-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i-(20|go|ma)|i230|iac( |-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|-[a-w])|libw|lynx|m1-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|-([1-8]|c))|phil|pire|pl(ay|uc)|pn-2|po(ck|rt|se)|prox|psio|pt-g|qa-a|qc(07|12|21|32|60|-[2-7]|i-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h-|oo|p-)|sdk\/|se(c(-|0|1)|47|mc|nd|ri)|sgh-|shar|sie(-|m)|sk-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h-|v-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl-|tdg-|tel(i|m)|tim-|t-mo|to(pl|sh)|ts(70|m-|m3|m5)|tx-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas-|your|zeto|zte-/i.test(a.substr(0,4))) check = true;})(navigator.userAgent||navigator.vendor);
  return check;
}


