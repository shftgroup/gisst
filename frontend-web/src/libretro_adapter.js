var Module =
    {
        noInitialRun: true,
        //canvas:document.getElementById("canvas"),
        preRun: [],
        postRun: [],
        print: function(text)
        {
            console.log(text);
        },
        printErr: function(text)
        {
            console.log(text);
        },
    };

function startRetroArch(canvas, arguments, initialized_cb)
{
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
    coreScript.addEventListener("load", loaded_cb);
    document.head.appendChild(coreScript);
}
