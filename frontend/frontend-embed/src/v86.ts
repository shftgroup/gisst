import {nested_replace,StringIndexable} from './util';
import {EmbedV86,StateInfo} from 'embedv86';
import {EmbedOptions,Environment, ColdStart, StateStart, ReplayStart, ObjectLink, SaveFileLink, EmuControls} from './types.d';


let v86_loading = false;
let v86_loaded = false;
const emulators:EmbedV86[] = [];

async function downloadScript(src:string) : Promise<Blob> {
  const resp = await fetch(src);
  const blob = await resp.blob();
  return blob;
}

function load_v86(gisst_root:string) : Promise<Blob | string> {
  const path = gisst_root+'/v86/libv86.js';
  if (gisst_root.startsWith("https://")) {
    return downloadScript(path);
  } else {
    return new Promise((resolve) => resolve(path));
  }
}

export async function init(gisst_root:string, environment:Environment, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[], _saves:SaveFileLink[], container:HTMLDivElement, _options:EmbedOptions):Promise<EmuControls> {
  if(!v86_loaded && !v86_loading) {
    v86_loading = true;
    console.log("Loading v86");
    let blobOrUrl = await load_v86(gisst_root);
    if (blobOrUrl instanceof Blob) {
      blobOrUrl = URL.createObjectURL(blobOrUrl);
    }
    const scpt = document.createElement("script");
    scpt.src = blobOrUrl;
    scpt.onload = () => {
      v86_loaded = true;
      v86_loading = false;
    }
    document.head.appendChild(scpt);
  }
  while(v86_loading) {
    console.log("Another v86 instance is loading, please wait...");
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
  console.log("v86 loaded");
  for (const obj of manifest) {
    if (obj.object_role == "content") {
      const obj_path = "storage/"+obj.file_dest_path;
      const idx = obj.object_role_index.toString();
      nested_replace(environment.environment_config as StringIndexable, "$CONTENT"+idx, obj_path);
      if (obj.object_role_index == 0) {
        nested_replace(environment.environment_config as StringIndexable, "$CONTENT", obj_path);
      }
    }
  }
  let entry_state:string|null = null;
  if (start.type == "state") {
    const data = (start as StateStart).data;
    entry_state = "storage/"+data.file_dest_path;
  }
  let movie:string|null = null;
  if (start.type == "replay") {
    const data = (start as ReplayStart).data;
    movie = "storage/"+data.file_dest_path;
  }

  const v86 = new EmbedV86({
    wasm_root:gisst_root+"/v86",
    bios_root:gisst_root+"/v86/bios",
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
  function click_to_activate() {
      activate(v86);
  }
  container.addEventListener(
    "click",
    click_to_activate
  );
  let is_muted = false;
  return {
    halt: async () => {
      container.removeEventListener("click", click_to_activate);
      v86.clear();
    },
    toggle_mute: () => {
      is_muted = !is_muted;
      v86.emulator.speaker_adapter.mixer.set_volume(is_muted ? 0 : 1, undefined)
    }
  };
}

function activate(v86:EmbedV86) {
  for(const emu of emulators) {
    if(emu.emulator) {
      emu.emulator.keyboard_set_status(emu === v86);
      emu.emulator.mouse_set_status(emu === v86);
      if (emu == v86) {
        emu.emulator.lock_mouse();
      }
    }
  }
}

export async function destroy(v86:EmbedV86) {
  emulators.splice(emulators.indexOf(v86),1);
  v86.clear();
}
