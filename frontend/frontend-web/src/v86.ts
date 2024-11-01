import {UI, GISSTDBConnector, GISSTModels, ReplayMode as UIReplayMode} from 'gisst-player';
import {saveAs, nested_replace} from './util';
import {EmbedV86,StateInfo} from 'embedv86';
import {Environment, ColdStart, StateStart, ReplayStart, ObjectLink, EmbedOptions} from './types.d';
import * as tus from 'tus-js-client';
let ui_state:UI;
let db:GISSTDBConnector;


export async function init(environment:Environment, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[], boot_into_record:boolean, _embed_options:EmbedOptions) {
  db = new GISSTDBConnector(window.location.protocol + "//" + window.location.host);
  for (const obj of manifest) {
    if (obj.object_role == "content") {
      const obj_path = "storage/"+obj.file_dest_path+"/"+obj.file_hash+"-"+obj.file_filename;
      const idx = obj.object_role_index.toString();
      nested_replace(environment.environment_config, "$CONTENT"+idx, obj_path);
      if (obj.object_role_index == 0) {
        nested_replace(environment.environment_config, "$CONTENT", obj_path);
      }
    }
  }
  let entry_state:string|null = null;
  let entry_screenshot:string|null = null;
  if (start.type == "state") {
    const data = (start as StateStart).data;
    entry_state = "storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename;
    if(!data.screenshot_id) {
      console.error("No screenshot for entry state");
      entry_screenshot = "";
    } else {
      const screenshot_data = (await db.getRecordById("screenshot", data.screenshot_id)) as GISSTModels.Screenshot;
      entry_screenshot = screenshot_data.screenshot_data;
    }
  }
  let movie:string|null = null;
  if (start.type == "replay") {
    const data = (start as ReplayStart).data;
    movie = "storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename;
  }

  const state_to_replay:Array<number|null> = [];

  /* eslint prefer-const: ["error", { ignoreReadBeforeAssign: true }], no-use-before-define: "error" */
  let v86:EmbedV86;
  let is_muted = false;
  ui_state = new UI(
    <HTMLDivElement>document.getElementById("ui")!,
    {
      "toggle_mute": () => {
        is_muted = !is_muted;
        v86.emulator.speaker_adapter.mixer.set_volume(is_muted ? 0 : 1, undefined);
      },
      "load_state":(n:number) => {
        // get the replay of state n
        // if it's not the same as the active replay we have to do something
        const replay = state_to_replay[n];
        if(replay !== v86.active_replay) {
          v86.stop_replay();
          if(replay !== null) {
            v86.play_replay_slot(replay);
          }
        }
        v86.load_state_slot(n);
      },
      "save_state":() => {
        v86.save_state()
      },
      "start_replay":() => {
        v86.record_replay()
      },
      "stop_and_save_replay":() => {
        v86.stop_replay()
      },
      "play_replay":(n:number) => v86.play_replay_slot(n),
      "download_file":(category:"state" | "save" | "replay", file_name:string) => {
        v86.download_file(category, file_name).then(([blob,name]) => saveAs(blob,name));
      },
      "checkpoints_of":(replay_slot:number) => {
        const rep = v86.replays[replay_slot].checkpoints;
        return rep.map((cp) => cp.name);
      },
      "upload_file":(category:"state" | "save" | "replay", file_name:string, metadata:GISSTModels.Metadata) => {
        return new Promise((resolve, reject) => {
          v86.download_file(category, file_name).then(([blob, name]) => {
            db.uploadFile(new File([blob], name),
              (error:Error) => { reject(error.message)},
              (_percentage:number) => {},
              (upload:tus.Upload) => {
                const url_parts = upload.url!.split('/');
                const uuid_string = url_parts[url_parts.length - 1];
                metadata.record.file_id = uuid_string;
                if(category == "state"){
                  db.uploadRecord({screenshot_data: metadata.screenshot.split(",")[1]}, "screenshot")
                    .then((screenshot:GISSTModels.DBRecord) => {
                      (metadata.record as GISSTModels.State).screenshot_id = (screenshot as GISSTModels.Screenshot).screenshot_id;
                      console.log(metadata);
                      db.uploadRecord(metadata.record, "state")
                        .then((state:GISSTModels.DBRecord) => {
                          (metadata.record as GISSTModels.State).state_id = (state as GISSTModels.State).state_id
                          resolve(metadata);
                        })
                        .catch(() => reject("State upload from v86 failed."))
                    })
                    .catch(() => reject("Screenshot upload from v86 failed."))
                }else {
                  db.uploadRecord(metadata.record, category)
                    .then((record:GISSTModels.DBRecord) => {
                      if (category === "replay") {
                        (metadata.record as GISSTModels.Replay).replay_id = (record as GISSTModels.Replay).replay_id;
                      } else {
                        (metadata.record as GISSTModels.Save).save_id = (record as GISSTModels.Save).save_id;
                      }
                      resolve(metadata)
                    })
                    .catch(() => reject(category + " upload from v86 failed"))
                }
              }
            )
              .catch(() => reject("File upload from v86 failed."))
          })
        })
      }
    },
    false,
      JSON.parse(document.getElementById("config")!.textContent!) as GISSTModels.FrontendConfig
  );
  const container = <HTMLDivElement>document.getElementById("canvas_div")!;
  v86 = new EmbedV86({
    wasm_root:"/v86",
    bios_root:"/v86/bios",
    record_from_start:boot_into_record,
    content_root:window.location.origin,
    container,
    register_replay:(nom:string)=> {
      if(movie && nom == "replay0") {
        const data = (start as ReplayStart).data;
        ui_state.newReplay(nom, "init", data);
      } else {
        ui_state.newReplay(nom, nom);
      }
      //ui_state.setReplayMode(UIReplayMode.Record);
    },
    stop_replay:()=>{
    },
    states_changed:(added:StateInfo[], removed:StateInfo[]) => {
      for(const si of removed) {
        ui_state.removeState(si.name);
      }
      for(const si of added) {
        if(entry_state && si.name == "state0") {
          si.thumbnail = entry_screenshot!;
          const data = (start as StateStart).data;
          ui_state.newState(si.name, si.thumbnail, "init", data);
          state_to_replay.push(v86.active_replay);
        } else {
          ui_state.newState(si.name,si. thumbnail, String(v86.active_replay));
          state_to_replay.push(v86.active_replay);
        }
      }
    },
    replay_checkpoints_changed:(added:StateInfo[], _removed:StateInfo[]) => {
      for(const si of added) {
        ui_state.newState(si.name,si.thumbnail,String(v86.active_replay));
        state_to_replay.push(v86.active_replay);
      }
    },
  });
  (<HTMLImageElement>document.getElementById("webplayer-preview")!).src = "/media/canvas-v86.svg";
  // document.getElementById("v86_controls")!.classList.remove("hidden");
  const prev = document.getElementById("webplayer-preview")!;
  prev.classList.add("loaded");
  prev.addEventListener(
    "click",
    async function () {
      const canv = <HTMLCanvasElement>document.getElementById("canvas")!;
      prev.classList.add("hidden");
      document.getElementById("webplayer-textmode")!.classList.remove("hidden");
      v86.run(environment.environment_config, entry_state, movie);
      canv.classList.remove("hidden");
      return false;
    });
  setInterval(() => {
    if(v86.active_replay === null) {
      ui_state.setReplayMode(UIReplayMode.Inactive);
    } else {
      ui_state.setReplayMode(v86.replays[v86.active_replay].mode as UIReplayMode);
    }
  }, 250);
  container.addEventListener("click", () => { v86.emulator.lock_mouse(); } )
}
