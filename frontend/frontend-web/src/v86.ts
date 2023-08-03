import {UI} from 'gisst-player';
import {saveAs, nested_replace} from './util';
import {EmbedV86,StateInfo} from 'embedv86';
import {Environment, ColdStart, StateStart, ReplayStart, ObjectLink} from './types';
let ui_state:UI;


export async function init(environment:Environment, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[]) {
  let content = manifest.find((o) => o.object_role=="content")!;
  let content_path = "storage/"+content.file_dest_path+"/"+content.file_hash+"-"+content.file_filename;
  nested_replace(environment.environment_config, "$CONTENT", content_path);
  let entry_state:string|null = null;
  if (start.type == "state") {
    let data = (start as StateStart).data;
    entry_state = "storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename;
  }
  let movie:string|null = null;
  if (start.type == "replay") {
    let data = (start as ReplayStart).data;
    movie = "storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename;
  }


  let v86:EmbedV86 = new EmbedV86({
    wasm_root:"/v86",
    bios_root:"/v86/bios",
    content_root:window.location.origin,
    container: <HTMLDivElement>document.getElementById("canvas_div")!,
    register_replay:(nom:string)=>ui_state.newReplay(nom),
    stop_replay:()=>{
      ui_state.clearCheckpoints();
    },
    states_changed:(added:StateInfo[], removed:StateInfo[]) => {
      for(let si of removed) {
        ui_state.removeState(si.name);
      }
      for(let si of added) {
        ui_state.newState(si.name,si.thumbnail);
      }
    },
    replay_checkpoints_changed:(added:StateInfo[], removed:StateInfo[]) => {
      for(let si of removed) {
        ui_state.removeCheckpoint(si.name);
      }
      for(let si of added) {
        ui_state.newCheckpoint(si.name,si.thumbnail);
      }
    },
  });
  ui_state = new UI(
    <HTMLDivElement>document.getElementById("ui")!,
    {
      "load_state":(n:number) => {
        if(v86.active_replay != null) { v86.stop_replay(); }
        v86.load_state_slot(n);
      },
      "play_replay":(n:number) => v86.play_replay_slot(n),
      "load_checkpoint":(n:number) => {
        if(v86.active_replay == null) { throw "Can't load checkpoint if no replay"; }
        v86.load_state_slot(n);
      },
      "download_file":(category:"state" | "save" | "replay", file_name:string) => {
        v86.download_file(category, file_name).then(([blob,name]) => saveAs(blob,name));
      }
    },
    false
  );

  document.getElementById("v86_controls")!.classList.remove("hidden");
  document.getElementById("v86_save")?.addEventListener("click",
    () => v86.save_state()
  );
  document.getElementById("v86_record")?.addEventListener("click",
    () => v86.record_replay()
  );
  document.getElementById("v86_stop")?.addEventListener("click",
    () => v86.stop_replay()
  );
  let prev = document.getElementById("webplayer-preview")!;
  prev.classList.add("loaded");
  prev.addEventListener(
    "click",
    async function () {
      let canv = <HTMLCanvasElement>document.getElementById("canvas")!;
      prev.classList.add("hidden");
      document.getElementById("webplayer-textmode")!.classList.remove("hidden");
      v86.run(environment.environment_config, entry_state, movie);
      canv.classList.remove("hidden");
      return false;
    });
}
