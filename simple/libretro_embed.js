/**
 * RetroArch Web Player
 *
 * This provides the basic JavaScript for the RetroArch web player.
 */
const core = "fceumm";
const content_folder = "/content/";
const content = "bfight.nes";
const entryState = true;
const movie = false;

const FS_CHECK_INTERVAL = 1000;

console.assert(!(entryState && movie), "It is invalid to have both an entry state and play back a movie");

var movieArg = "-R";
if (movie) {
    movieArg = "-P";
}
var contentBase = content.substr(0,content.lastIndexOf("."));

var retro_args =  ["-v"];
if (entryState) {
    retro_args.push("-e");
    retro_args.push("1");
}
if (movie) {
    retro_args.push("-P");
    retro_args.push("/home/web_user/content/movie.bsv");
} else {
    retro_args.push("-R");
    retro_args.push("/home/web_user/retroarch/userdata/movie.bsv");
}
retro_args.push("/home/web_user/content/"+content);

var BrowserFS = BrowserFS;
var afs;
var initializationCount = 0;

function cleanupStorage()
{
    localStorage.clear();
    document.getElementById("btnClean").disabled = true;
}

function fsInit()
{
    afs = new BrowserFS.FileSystem.InMemory();
    setupFileSystem("browser");
    appInitialized();
}

function appInitialized()
{
    /* Need to wait for both the file system and the wasm runtime 
       to complete before enabling the Run button. */
    initializationCount++;
    if (initializationCount == 2)
    {
        preLoadingComplete();
    }
}

function preLoadingComplete()
{
    /* Make the Preview image clickable to start RetroArch. */
    document.getElementById("webplayer-preview").classList.add("loaded");
    document.getElementById("webplayer-preview").addEventListener(
        "click",
        function () {
            startRetroArch();
            return false;
        });
}

var renderedSaves = [];
function checkChangedSaves() {
    try {
        var newSaves = FS.readdir("/home/web_user/retroarch/userdata/states");
        // if any new ones, update lastSaves
        for(var i = renderedSaves.length; i < newSaves.length; i++) {
            let save = newSaves[i];
            console.log(save);
            renderedSaves.push(save);
        }
    } catch(e) {
        if (e instanceof FS.ErrnoError) {
            // do nothing
        } else {
            throw e;
        }
    }
}

function setupFileSystem(backend)
{
    /* create a mountable filesystem that will server as a root
       mountpoint for browserfs */
    var mfs =  new BrowserFS.FileSystem.MountableFileSystem();

    /* create an XmlHttpRequest filesystem for the bundled data */
    var xfs1 =  new BrowserFS.FileSystem.XmlHttpRequest
    (".index-xhr", "assets/frontend/bundle/");
    /* create an XmlHttpRequest filesystem for core assets */
    // var xfs2 =  new BrowserFS.FileSystem.XmlHttpRequest
    // ([], "assets/cores/");
    var xfs_content_files = {"retroarch.cfg":null};
    xfs_content_files[content] = null;
    if (entryState) {
        xfs_content_files["entry_state"] = null;
    }
    if (movie) {
        xfs_content_files["movie.bsv"] = null;
    }
    var xfs_content = new BrowserFS.FileSystem.XmlHttpRequest(xfs_content_files, content_folder);

    console.log("WEBPLAYER: initializing filesystem: " + backend);
    mfs.mount('/home/web_user/retroarch/userdata', afs);
    setInterval(checkChangedSaves, FS_CHECK_INTERVAL);

    mfs.mount('/home/web_user/retroarch/bundle', xfs1);
    // mfs.mount('/home/web_user/retroarch/userdata/content/downloads', xfs2);
    mfs.mount('/home/web_user/content', xfs_content);
    BrowserFS.initialize(mfs);
    var BFS = new BrowserFS.EmscriptenFS();
    FS.mount(BFS, {root: '/home'}, '/home');

    if (entryState) {
        FS.mkdir("/home/web_user/retroarch/userdata/states");
        copyFile("/home/web_user/content/entry_state",
                 "/home/web_user/retroarch/userdata/states/"+contentBase+".state1.entry");
    }
    copyFile("/home/web_user/content/retroarch.cfg", "/home/web_user/retroarch/userdata/retroarch.cfg");
    console.log("WEBPLAYER: " + backend + " filesystem initialization successful");
}

function copyFile(from, to) {
    var buf = FS.readFile(from);
    FS.writeFile(to, buf);
}

function startRetroArch()
{
    document.getElementById("canvas").classList.remove("hidden");
    document.getElementById("webplayer-preview").classList.add("hidden");

    Module['callMain'](Module['arguments']);
    Module['resumeMainLoop']();
    document.getElementById('canvas').focus();
}

var Module =
    {
        noInitialRun: true,
        arguments: retro_args,
        preRun: [],
        postRun: [],
        onRuntimeInitialized: function()
        {
            appInitialized();
        },
        print: function(text)
        {
            console.log(text);
        },
        printErr: function(text)
        {
            console.log(text);
        },
        canvas: document.getElementById('canvas'),
        totalDependencies: 0,
        monitorRunDependencies: function(left)
        {
            this.totalDependencies = Math.max(this.totalDependencies, left);
        }
    };

function domLoaded() {
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

    if (!core) {
        core = 'gambatte';
    }
    
    // Load the Core's related JavaScript.
    var coreScript = document.createElement('script');
    coreScript.type = 'text/javascript';
    coreScript.src = core+'_libretro.js';
    coreScript.addEventListener("error", function() {console.error("Couldn't load core file", core+"_libretro.js");});
    coreScript.addEventListener("load", function() {
                fsInit();
        });
    
    document.head.appendChild(coreScript);
}

// When the browser has loaded everything.
if (document.readyState == 'loading') {
    document.addEventListener("DOMContentLoaded", domLoaded);
} else {
    domLoaded();
}

/* Simulate a key event */
function keyPress(k)
{
    kp(k, "keydown");
    setTimeout(function(){kp(k, "keyup");}, 50);
}

kp = function(k, event) {
    var oEvent = new KeyboardEvent(event, { code: k });

    document.dispatchEvent(oEvent);
    document.getElementById('canvas').focus();
}
