import {EmbedV86,StateInfo} from 'embedv86';
import {EmbedOptions, Environment, Work, Instance, ColdStart, StateStart, ReplayStart, CoreFileLink, ObjectLink, SaveFileLink, EmuControls, StringIndexable} from './types.d';

const emulators:EmbedV86[] = [];

export function nested_replace(obj:StringIndexable, target:string, replacement:string) {
  for(const key in obj) {
    if(typeof(obj[key]) == "object") {
      nested_replace(obj[key] as StringIndexable, target, replacement);
    } else if(obj[key] == target) {
      obj[key] = replacement;
    }
  }
}

async function loadScript(url:string) {
  return new Promise((resolve, reject) => {
    const ourScript = document.createElement('script');
    ourScript.addEventListener('load', (evt) => {
      resolve(evt);
    });
    ourScript.addEventListener('error', (reason) => {
      reject(reason);
    });
     // add your script's src here
    ourScript.src = url;
    document.head.appendChild(ourScript);
  });
}

export async function init(gisst_root:string, environment:Environment, work:Work, instance:Instance, start:ColdStart | StateStart | ReplayStart, core_manifest:CoreFileLink[], manifest:ObjectLink[], _saves:SaveFileLink[], container:HTMLDivElement, options:EmbedOptions):Promise<EmuControls> {
  let v86_wasm = null;
  for (const obj of core_manifest) {
    if (obj.core_role == "dependency") {
      const filename = obj.file_filename;
      const obj_path = "storage/"+obj.file_dest_path;
      if (filename == "v86.wasm") {
        v86_wasm = obj_path;
      } else {
        nested_replace(environment.environment_config as StringIndexable, filename, obj_path);
      }
    } else if (obj.core_role == "entrypoint") {
        // libv86.js
        await loadScript(gisst_root+"/storage/"+obj.file_dest_path);
    }
  }
  if (!v86_wasm) { throw "No v86 wasm path defined"; }
  for (const obj of manifest) {
    if (obj.object_role == "content") {
      const obj_path = "storage/"+obj.file_dest_path;
      const idx = obj.object_role_index.toString();
      nested_replace(environment.environment_config, "$CONTENT"+idx, obj_path);
      if (obj.object_role_index == 0) {
        nested_replace(environment.environment_config, "$CONTENT", obj_path);
      }
    }
  }
  let use_graphical_text = true;
  let entry_state:string|null = null;
  if (start.type == "state") {
    const data = (start as StateStart).data;
    // Compatibility Note 1: Before this time, and while we were still
    // using ambient cores, states and replays output by v86 did not
    // include all the VGA memory they need to in order to recreate
    // the graphical text display properly.
    if (new Date(data.created_on) < new Date('2025-05-01')) {
      use_graphical_text = false;
    }
    entry_state = "storage/"+data.file_dest_path;
  }
  let movie:string|null = null;
  if (start.type == "replay") {
    const data = (start as ReplayStart).data;
    // See Compatibility Note 1
    if (new Date(data.created_on) < new Date('2025-05-01')) {
      use_graphical_text = false;
    }
    movie = "storage/"+data.file_dest_path;
  }

  const v86 = new EmbedV86({
    wasm_file:gisst_root+"/"+v86_wasm,
    bios_root:gisst_root,
    record_from_start:options.record_from_start,
    content_root:gisst_root,
    container,
    use_graphical_text,
    record_video:options.record_video,
    register_replay:(_nom:string)=>{},
    stop_replay:()=>{},
    states_changed:(_added:StateInfo[], _removed:StateInfo[]) => {},
    replay_checkpoints_changed:(_added:StateInfo[], _removed:StateInfo[]) => {},
  });
  emulators.push(v86);
  const self = {
    halt: async () => {
      container.removeEventListener("click", click_to_activate);
      v86.clear();
    },
    toggle_mute: () => {
      is_muted = !is_muted;
      v86.emulator.speaker_adapter.mixer.set_volume(is_muted ? 0 : 1, undefined)
    },
    gisst_root,
    environment,
    work,
    instance,
    start,
    saves:_saves,
    core_manifest,
    manifest,
    container,
    embed_options:options,
    on_ready: () => {},
    inner:v86
  };
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
      self.on_ready();
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
  return self;
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
