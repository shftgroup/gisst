/* eslint-env browser */

export interface LibretroModule extends EmscriptenModule, LibretroModuleDef {
  canvas:HTMLCanvasElement;
  callMain(args:string[]): void;
  // resumeMainLoop(): void;
  EmscriptenSendCommand(msg:string):void;
  EmscriptenReceiveCommandReply():string;
  cwrap: typeof cwrap;
}
interface LibretroModuleDef {
  startRetroArch(canvas:HTMLCanvasElement, args:string[], initialized_cb:() => void):void;
  locateFile(path:string,prefix:string):string;
  retroArchSend(msg:string):void;
  retroArchRecv():string|undefined;
  ENV:Environment,
  mainScriptUrlOrBlob: Blob | string;
  noInitialRun: boolean;
  noImageDecoding: boolean;
  noAudioDecoding: boolean;
  preRun: Array<{ (mod:object|undefined): void }>;
  postRun: Array<{ (mod:object|undefined): void }>;
  onRuntimeInitialized(): void;
  printErr(str:string):void;
}

const cores:Record<string,(mod:LibretroModuleDef) => Promise<LibretroModule>> = {};

async function downloadScript(src:string) : Promise<Blob> {
  const resp = await fetch(src);
  const blob = await resp.blob();
  return blob;
}

let setupWorker:Worker|null = null;
let filesystem_ready:boolean = false;
export interface SetupResponse {
  command:string,
  time:string
}


export function loadRetroArch(gisst_root:string, core:string, env:Environment, download_asset_bundle:boolean, loaded_cb:(mod:LibretroModule) => void) {
  if(download_asset_bundle) {
    if('OPFS' in env) {
      if(!setupWorker) {
        setupWorker = new Worker(new URL('./libretro.worker.ts', import.meta.url), {type:"module"});
        setupWorker.onmessage = (msg:MessageEvent<SetupResponse>) => {
          if(msg.data.command == "loaded_bundle") {
            filesystem_ready = true;
            localStorage.setItem("asset_time", msg.data.time);
          }
        }
        setupWorker.postMessage({command:"load_bundle",gisst_root,time:localStorage.getItem("asset_time") ?? ""});
      }
    } else {
      throw "Asset downloads currently require OPFS env variable set to /home/web_user/retroarch/";
    }
  }

  /**
   * Attempt to disable some default browser keys.
   */
  const keys:Record<number, string> = {
    9: "tab",
    13: "enter",
    16: "shift",
    18: "alt",
    27: "esc",
    33: "rePag",
    34: "avPag",
    35: "end",
    36: "home",
    37: "left",
    38: "up",
    39: "right",
    40: "down",
    112: "F1",
    113: "F2",
    114: "F3",
    115: "F4",
    116: "F5",
    117: "F6",
    118: "F7",
    119: "F8",
    120: "F9",
    121: "F10",
    122: "F11",
    123: "F12"
  };
  window.addEventListener('keydown', function (e:KeyboardEvent) {
    if (keys[e.which]) {
      e.preventDefault();
    }
  });
  const fsready:Promise<void> = new Promise((resolve) => {
    const check = () => {
      if (filesystem_ready || !download_asset_bundle) {
        resolve();
      } else {
        setTimeout(check, 500);
      }
    }
    check();
  });
  Promise.all([downloadScript(gisst_root+'/cores/'+core+'_libretro.js'),fsready]).then(([scriptBlob,_]) => {
    let initial_mod:LibretroModule | undefined;
    const module:LibretroModuleDef = {
      startRetroArch: function(canvas:HTMLCanvasElement, retro_args:string[], initialized_cb:() => void) {
        const me = <LibretroModule>this;
        if(!canvas.tabIndex) { canvas.tabIndex = 1; }
        canvas.addEventListener("click", () => canvas.focus());
        me.canvas = canvas;
        me.ENV = env;
	me.callMain(retro_args);
        initialized_cb();
        canvas.focus();
      },
      retroArchSend: function(msg:string) {
        const me = <LibretroModule>this;
        me.EmscriptenSendCommand(msg);
      },
      retroArchRecv: function() {
        const me = <LibretroModule>this;
        return me.EmscriptenReceiveCommandReply();
      },
      ENV: env,
      noInitialRun: true,
      noImageDecoding: true,
      noAudioDecoding: true,
      preRun: [
        function(init_mod:object|undefined) {
          const module = <LibretroModule>(init_mod!);
          for (const [k,v] of Object.entries(env)) {
            module.ENV[k] = v;
          }
        },
        function(init_mod:object|undefined) {
          if(init_mod === undefined) { throw "Must use modularized emscripten"; }
          initial_mod = <LibretroModule>(init_mod!);
        },
      ],
      postRun:[],
      onRuntimeInitialized: function() {
        if(initial_mod === undefined) { throw "Must use modularized emscripten libretro"; }
        const module = <LibretroModule>(initial_mod!);
        loaded_cb(module);
      },
      locateFile: function(path, _prefix) {
        return gisst_root+'/cores/'+path;
      },
      printErr: function(text:string) {
        console.log(text);
      },
      mainScriptUrlOrBlob: scriptBlob
    };
    function instantiate(core_factory:(mod:LibretroModuleDef) => Promise<LibretroModule>) {
      core_factory(module).catch(err => {
        console.error("Couldn't instantiate module", err);
        throw err;
      });
    }
    if (core in cores) {
      instantiate(cores[core]);
    } else {
      /* TODO use new URL, import.meta, etc instead of blob */
      import(/* @vite-ignore */ URL.createObjectURL(scriptBlob)).then(fac => {
        cores[core] = fac.default;
        instantiate(cores[core]);
      }).catch(err => { console.error("Couldn't instantiate module", err); throw err; });
    }
  });
}
