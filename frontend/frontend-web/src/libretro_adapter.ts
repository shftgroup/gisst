/* eslint-env browser */

export interface LibretroModule extends EmscriptenModule, LibretroModuleDef {
  canvas:HTMLCanvasElement;
  callMain(args:string[]): void;
  resumeMainLoop(): void;
}
interface LibretroModuleDef {
  startRetroArch(canvas:HTMLCanvasElement, args:string[], initialized_cb:() => void):void;
  retroArchSend(msg:string):void;
  retroArchRecv():string|undefined;
  message_queue:[Uint8Array,number][];
  message_out:string[];
  message_accum:string;
  encoder:TextEncoder;
  noInitialRun: boolean;
  preRun: Array<{ (mod:object|undefined): void }>;
  postRun: Array<{ (mod:object|undefined): void }>;
  printErr(str:string):void;
}

const cores:Record<string,(mod:LibretroModuleDef) => Promise<LibretroModule>> = {};

export function loadRetroArch(gisst_root:string, core:string, loaded_cb:(mod:LibretroModule) => void) {
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
    const module:LibretroModuleDef = {
        startRetroArch: function(canvas:HTMLCanvasElement, retro_args:string[], initialized_cb:() => void) {
            const me = <LibretroModule>this;
            if(!canvas.tabIndex) { canvas.tabIndex = 1; }
            canvas.addEventListener("click", () => canvas.focus());
            me.canvas = canvas;
            me.arguments = retro_args;
            me.callMain(me.arguments);
            me.resumeMainLoop();
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
        noInitialRun: true,
        preRun: [
            function(init_mod:object|undefined) {
                if(init_mod === undefined) { throw "Must use modularized emscripten"; }
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
        postRun: [],
        printErr: function(text:string) {
            console.log(text);
        }
    };
    function instantiate(core_factory:(mod:LibretroModuleDef) => Promise<LibretroModule>) {
        core_factory(module).then(loaded_cb).catch(err => {
            console.error("Couldn't instantiate module", err);
            throw err;
        });
    }
    if (core in cores) {
        instantiate(cores[core]);
    } else {
        import(/* @vite-ignore */ gisst_root+'/cores/'+core+'_libretro.js').then(fac => {
            cores[core] = fac.default;
            instantiate(cores[core]);
        }).catch(err => { console.error("Couldn't instantiate module", err); throw err; });
    }
}
