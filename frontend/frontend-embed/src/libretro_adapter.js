/* eslint-env browser */

function loadRetroArch(core, loaded_cb) {
    /**
     * Attempt to disable some default browser keys.
     */
    var keys = {
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
    window.addEventListener('keydown', function (e) {
        if (keys[e.which]) {
            e.preventDefault();
        }
    });
    let module = {
        startRetroArch: function(canvas, retro_args, initialized_cb) {
            this['canvas'] = canvas;
            this['arguments'] = retro_args;
            this['callMain'](this['arguments']);
            this['resumeMainLoop']();
            initialized_cb();
            canvas.focus();
        },

        encoder: new TextEncoder(),
        message_queue:[],
        message_out:[],
        message_accum:"",
          
        retroArchSend: function(msg) {
            let bytes = this.encoder.encode(msg+"\n");
            this.message_queue.push([bytes,0]);
        },
        retroArchRecv: function() {
            let out = this.message_out.shift();
            if(out == null && this.message_accum != "") {
                out = this.message_accum;
                this.message_accum = "";
            }
            return out;
        },
        noInitialRun: true,
        preRun: [
            function(module) {
                function stdin() {
                    // Return ASCII code of character, or null if no input
                    while(module.message_queue.length > 0){
                        let [msg,index] = module.message_queue[0];
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
                function stdout(c) {
                    if(c == null) {
                        // flush
                        if(module.message_accum != "") {
                            module.message_out.push(module.message_accum);
                            module.message_accum = "";
                        }
                    } else {
                        let s = String.fromCharCode(c);
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
                module.FS.init(stdin,stdout);
            },
        ],
        postRun: [],
        printErr: function(text) {
            console.log(text);
        },
    };
    let core_module = await import('/cores/'+core+"_libretro.js");
    core_module.default(module).then(loaded_cb).catch(err => { console.err("Couldn't instantiate module", err); throw err });
}
