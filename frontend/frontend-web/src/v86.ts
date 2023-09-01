import {UI, UIIDConst, GISSTDBConnector, GISSTModels} from 'gisst-player';
import {saveAs, nested_replace} from './util';
import {EmbedV86,StateInfo} from 'embedv86';
import {Environment, ColdStart, StateStart, ReplayStart, ObjectLink} from './types';
import * as tus from 'tus-js-client';
let ui_state:UI;
let db:GISSTDBConnector;


export async function init(environment:Environment, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[]) {
  const content = manifest.find((o) => o.object_role=="content")!;
  const content_path = "storage/"+content.file_dest_path+"/"+content.file_hash+"-"+content.file_filename;
  nested_replace(environment.environment_config, "$CONTENT", content_path);
  let entry_state:string|null = null;
  if (start.type == "state") {
    const data = (start as StateStart).data;
    entry_state = "storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename;
  }
  let movie:string|null = null;
  if (start.type == "replay") {
    const data = (start as ReplayStart).data;
    movie = "storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename;
  }


  let v86:EmbedV86;
  db = new GISSTDBConnector(window.location.protocol + "//" + window.location.host);
  ui_state = new UI(
    <HTMLDivElement>document.getElementById("ui")!,
    {
        "load_state":(n:number) => {
        if(v86.active_replay != null) { v86.stop_replay(); }
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
      "load_checkpoint":(n:number) => {
        if(v86.active_replay == null) { throw "Can't load checkpoint if no replay"; }
        v86.load_state_slot(n);
      },
      "download_file":(category:"state" | "save" | "replay", file_name:string) => {
        v86.download_file(category, file_name).then(([blob,name]) => saveAs(blob,name));
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
                                db.uploadRecord({screenshot_id:"00000000-0000-0000-0000-000000000000", screenshot_data: metadata.screenshot.split(",")[1]}, "screenshot")
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

  v86 = new EmbedV86({
    wasm_root:"/v86",
    bios_root:"/v86/bios",
    content_root:window.location.origin,
    container: <HTMLDivElement>document.getElementById("canvas_div")!,
    register_replay:(nom:string)=>ui_state.newReplay(nom),
    stop_replay:()=>{
      ui_state.clearCheckpoints();
    },
    states_changed:(added:StateInfo[], removed:StateInfo[]) => {
      for(const si of removed) {
        ui_state.removeState(si.name);
      }
      for(const si of added) {
        ui_state.newState(si.name,si.thumbnail);
      }
    },
    replay_checkpoints_changed:(added:StateInfo[], removed:StateInfo[]) => {
      for(const si of removed) {
        ui_state.removeCheckpoint(si.name);
      }
      for(const si of added) {
        ui_state.newCheckpoint(si.name,si.thumbnail);
      }
    },
  });
  (<HTMLImageElement>document.getElementById("webplayer-preview")!).src = "/media/canvas-v86.png";
  // document.getElementById("v86_controls")!.classList.remove("hidden");
  document.getElementById(UIIDConst.EMU_START_REPLAY_BUTTON)?.addEventListener("click",
    () => v86.record_replay()
  );
  document.getElementById(UIIDConst.EMU_FINISH_REPLAY_BUTTON)?.addEventListener("click",
    () => v86.stop_replay()
  );
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
}
