/* eslint-env browser */
export interface LibretroModule extends EmscriptenModule, LibretroModuleDef {
  canvas:HTMLCanvasElement;
  callMain(args:string[]): void;
  // resumeMainLoop(): void;
}
interface LibretroModuleDef {
  startRetroArch(canvas:HTMLCanvasElement, args:string[], initialized_cb:() => void):void;
  locateFile(path:string,prefix:string):string;
  retroArchSend(msg:string):void;
  retroArchRecv():string|undefined;
  message_queue:[Uint8Array,number][];
  message_out:string[];
  message_accum:string;
  ENV:Environment,
  mainScriptUrlOrBlob: Blob | string;
  encoder:TextEncoder;
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
    let initial_mod:object | undefined;
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

      encoder: new TextEncoder(),
      message_queue:[],
      message_out:[],
      message_accum:"",

      retroArchSend: function(msg:string) {
        const bytes = this.encoder.encode(msg+"\n");
        this.message_queue.push([bytes,0]);
      },
      retroArchRecv: function() {
        let out:string | undefined = this.message_out.shift();
        if(out == null && this.message_accum != "") {
          out = this.message_accum;
          this.message_accum = "";
        }
        return out;
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
          initial_mod = init_mod;
          const module = <LibretroModule>(init_mod!);
          function stdin() {
            // Return ASCII code of character, or null if no input
            while(module.message_queue.length > 0){
              const [msg,index] = module.message_queue[0];
              if(index >= msg.length) {
                module.message_queue.shift();
              } else {
                module.message_queue[0][1] = index+1;
                // assumption: msg is a uint8array
                return msg[index];
              }
            }
            return null;
          }
          function stdout(c:number) {
            if(c == null) {
              // flush
              if(module.message_accum != "") {
                module.message_out.push(module.message_accum);
                module.message_accum = "";
              }
            } else {
              const s = String.fromCharCode(c);
              if(s == "\n") {
                if(module.message_accum != "") {
                  module.message_out.push(module.message_accum);
                  module.message_accum = "";
                }
              } else {
                module.message_accum = module.message_accum+s;
              }
            }
          }
          module.FS.init(stdin,stdout,null);
        },
      ],
      postRun:[],
      onRuntimeInitialized: function() {
        if(initial_mod === undefined) { throw "Must use modularized emscripten libretro"; }
        const module = <LibretroModule>(initial_mod!);
        module.ENV = env;
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
