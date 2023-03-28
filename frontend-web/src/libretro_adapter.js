const encoder = new TextEncoder();
const message_queue = [];
const message_out = [];
let message_accum = "";

function retroArchSend(msg) {
  let bytes = encoder.encode(msg+"\n");
  message_queue.push([bytes,0]);
}
function retroArchRecv() {
  let out = message_out.shift();
  if(out == null && message_accum != "") {
    out = message_accum;
    message_accum = "";
  }
  return out;
}

var Module =
  {
    noInitialRun: true,
    //canvas:document.getElementById("canvas"),
    preRun: [
      function() {
        function stdin() {
          // Return ASCII code of character, or null if no input
          while(message_queue.length > 0){
            let [msg,index] = message_queue[0];
            if(index >= msg.length) {
              message_queue.shift();
            } else {
              message_queue[0][1] = index+1;
              // assumption: msg is a uint8array
              return msg[index];
            }
          }
          return null;
        }
        function stdout(c) {
          if(c == null) {
            // flush
            if(message_accum != "") {
              message_out.push(message_accum);
              message_accum = "";
            }
          } else {
            let s = String.fromCharCode(c);
            if(s == "\n") {
              if(message_accum != "") {
                message_out.push(message_accum);
                message_accum = "";
              }
            } else {
              message_accum = message_accum+s;
            }
          }
        }
        FS.init(stdin,stdout);
      }
    ],
    postRun: [],
    printErr: function(text)
      {
        console.log(text);
      },
  };

function startRetroArch(canvas, arguments, initialized_cb) {
  Module['canvas'] = canvas;
  Module['arguments'] = arguments;
  Module['callMain'](Module['arguments']);
  Module['resumeMainLoop']();
  initialized_cb();
  canvas.focus();
}

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

  // Load the Core's related JavaScript.
  var coreScript = document.createElement('script');
  coreScript.type = 'text/javascript';
  coreScript.src = "cores/"+core+'_libretro.js';
  coreScript.addEventListener("error", function() {console.error("Couldn't load core file", coreScript.src);});
  coreScript.addEventListener("load", function() {Module.preRun.push(loaded_cb);});
  document.head.appendChild(coreScript);
}
