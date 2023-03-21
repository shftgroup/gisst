import {UI} from 'gisst-player';
import {saveAs} from './util';
import {EmbedV86} from 'embedv86';

let ui_state:UI;
export async function init(content_folder:string, content:string, entry_state:boolean, movie:boolean) {
  let v86:EmbedV86 = new EmbedV86({wasm_root:"v86",bios_root:"v86/bios", content_root:content_folder, container: <HTMLDivElement>document.getElementById("canvas_div")!, record_replay:(nom:string)=>ui_state.newReplay(nom), save_state:(nom:string, thumb:string) => ui_state.newState(nom,thumb)});
  ui_state = new UI(
    <HTMLDivElement>document.getElementById("states")!,
    <HTMLDivElement>document.getElementById("replays")!,
    <HTMLDivElement>document.getElementById("saves")!,
    {
      "load_state":(n:number) => v86.load_state_slot(n),
      "play_replay":(n:number) => v86.play_replay_slot(n),
      "download_file":(category:"state" | "save" | "replay", file_name:string) => {
        let [blob, name] = v86.download_file(category, file_name);
        saveAs(blob, name)
      }
    }
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
      v86.run(content, entry_state, movie);
      canv.classList.remove("hidden");
      return false;
    });
}
