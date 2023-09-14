import {nested_replace,StringIndexable} from './util';
import {EmbedV86,StateInfo} from 'embedv86';
import {Environment, ColdStart, StateStart, ReplayStart, ObjectLink} from './types';

let v86_loading = false;
let v86_loaded = false;
const emulators:EmbedV86[] = [];

function load_v86(gisst_root:string) : Promise<void> {
  return new Promise((resolve) => {
    v86_loading = true;
    const v86 = document.createElement("script");
    v86.onload = () => {
      v86_loading = false;
      v86_loaded = true;
      resolve();
    };
    v86.onerror = (err) => {
      console.error("Couldn't load v86", err);
      throw err;
    };
    v86.crossOrigin = "anonymous";
    v86.src = gisst_root+"/v86/libv86.js";
    document.head.appendChild(v86);
  });
}

export async function init(gisst_root:string, environment:Environment, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[], container:HTMLDivElement):Promise<EmbedV86> {
  if(!v86_loaded) {
    console.log("Loading v86");
    await load_v86(gisst_root);
  }
  while(v86_loading) {
    console.log("Another v86 instance is loading, please wait...");
    await new Promise(resolve => setTimeout(resolve, 2000));
  }
  console.log("v86 loaded");
  const content = manifest.find((o) => o.object_role=="content")!;
  const content_path = "storage/"+content.file_dest_path+"/"+content.file_hash+"-"+content.file_filename;
  nested_replace(environment.environment_config as StringIndexable, "$CONTENT", content_path);
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

  const v86 = new EmbedV86({
    wasm_root:"/v86",
    bios_root:"/v86/bios",
    record_from_start:false,
    content_root:gisst_root,
    container: container,
    register_replay:(_nom:string)=>{},
    stop_replay:()=>{
    },
    states_changed:(_added:StateInfo[], _removed:StateInfo[]) => {
    },
    replay_checkpoints_changed:(_added:StateInfo[], _removed:StateInfo[]) => {
    },
  });
  emulators.push(v86);
  const preview = container.getElementsByTagName("img")[0];
  preview.classList.add("gisst-embed-loaded");
  preview.addEventListener(
    "click",
    async function () {
      const canv = <HTMLCanvasElement>container.getElementsByTagName("canvas")[0]!;
      preview.classList.add("gisst-embed-hidden");
      canv.classList.remove("gisst-embed-hidden");
      container.getElementsByTagName("div")[0]!.classList.remove("gisst-embed-hidden");
      await v86.run(environment.environment_config, entry_state, movie);
      activate(v86);
      return false;
    });
  container.addEventListener(
    "click",
    function () {
      activate(v86);
    }
  );
  return v86;
}

function activate(v86:EmbedV86) {
  for(const emu of emulators) {
    if(emu.emulator) {
      emu.emulator.keyboard_set_status(emu === v86);
    }
  }
}

export async function destroy(v86:EmbedV86) {
  emulators.splice(emulators.indexOf(v86),1);
  v86.clear();
}
