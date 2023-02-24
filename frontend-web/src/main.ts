import './style.css'
import * as fetch from './fetchfs'
import {UI} from 'gisst-player';

const core = "fceumm";
const content_folder = "/content/";
const content = "bfight.nes";
const entryState = true;
const movie = false;

const FS_CHECK_INTERVAL = 1000;

const state_dir = "/home/web_user/retroarch/userdata/states";
const saves_dir = "/home/web_user/retroarch/userdata/saves";

console.assert(!(entryState && movie), "It is invalid to have both an entry state and play back a movie");

let content_base = content.substring(0, content.lastIndexOf("."));

let retro_args = ["-v"];
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
retro_args.push("--appendconfig");
retro_args.push("/home/web_user/content/retroarch.cfg");
retro_args.push("/home/web_user/content/" + content);


loadRetroArch(core,
  function () {
    let p1 = fetch.registerFetchFS(("assets/frontend/bundle/.index-xhr"), "assets/frontend/bundle", "/home/web_user/retroarch/bundle", true);
    let xfs_content_files: fetch.Index = { "retroarch.cfg": null };
    xfs_content_files[content] = null;
    if (entryState) {
      xfs_content_files["entry_state"] = null;
    }
    if (movie) {
      xfs_content_files["movie.bsv"] = null;
    }
    let p2 = fetch.registerFetchFS(xfs_content_files, content_folder, "/home/web_user/content", false);
    let p3 = fetch.registerFetchFS({"retroarch_web_base.cfg":null}, "assets", "/home/web_user/retroarch/", false);
    Promise.all([p1, p2, p3]).then(function () {
      fetch.mkdirp("/home/web_user/retroarch/userdata");
      copyFile("/home/web_user/retroarch/retroarch_web_base.cfg", "/home/web_user/retroarch/userdata/retroarch.cfg");
      if (entryState) {
        fetch.mkdirp(state_dir);
        copyFile("/home/web_user/content/entry_state",
          state_dir + "/" + content_base + ".state1.entry");
      }
      retroReady();
    });
  });

function copyFile(from: string, to: string): void {
  let buf = FS.readFile(from);
  FS.writeFile(to, buf);
}
let ui_state:UI;
// TODO add clear button to call ui_state.clear()
function retroReady(): void {
  ui_state = new UI(
    <HTMLDivElement>document.getElementById("states")!,
    <HTMLDivElement>document.getElementById("saves")!,
    {
      "load_state":(num:number) => load_state_slot(num),
      "download_file":(category:"state" | "save" | "movie", file_name:string) => {
        let path = "/home/web_user/retroarch/userdata";
        if(category == "state") {
          path += "/states";
        } else if(category == "save") {
          path += "/saves";
        } else if(category == "movie") {
          path += "/saves";
        } else {
          console.error("Invalid save category",category,file_name);
        }
        const data = FS.readFile(path+"/"+file_name);
        saveAs(new Blob([data]), file_name);
      }
    }
  );

  let prev = document.getElementById("webplayer-preview")!;
  prev.classList.add("loaded");
  prev.addEventListener(
    "click",
    function () {
      let canv = <HTMLCanvasElement>document.getElementById("canvas")!;
      prev.classList.add("hidden");
      startRetroArch(canv, retro_args, function () {
        let canv = document.getElementById("canvas")!;
        setInterval(checkChangedStatesAndSaves, FS_CHECK_INTERVAL);
        canv.classList.remove("hidden");
      });
      return false;
    });
}

function load_state_slot(n:number) {
  console.error("NYI",n);
}

// From MDN
/* Base64 string to array encoding */
function uint6ToB64(nUint6:number) {
  return nUint6 < 26
    ? nUint6 + 65
    : nUint6 < 52
    ? nUint6 + 71
    : nUint6 < 62
    ? nUint6 - 4
    : nUint6 === 62
    ? 43
    : nUint6 === 63
    ? 47
    : 65;
}
function base64EncArr(aBytes:Uint8Array) {
  let nMod3 = 2;
  let sB64Enc = "";

  const nLen = aBytes.length;
  let nUint24 = 0;
  for (let nIdx = 0; nIdx < nLen; nIdx++) {
    nMod3 = nIdx % 3;
    // To break your base64 into several 80-character lines, add:
    //   if (nIdx > 0 && ((nIdx * 4) / 3) % 76 === 0) {
    //      sB64Enc += "\r\n";
    //    }

    nUint24 |= aBytes[nIdx] << ((16 >>> nMod3) & 24);
    if (nMod3 === 2 || aBytes.length - nIdx === 1) {
      sB64Enc += String.fromCodePoint(
        uint6ToB64((nUint24 >>> 18) & 63),
        uint6ToB64((nUint24 >>> 12) & 63),
        uint6ToB64((nUint24 >>> 6) & 63),
        uint6ToB64(nUint24 & 63)
      );
      nUint24 = 0;
    }
  }
  return (
    sB64Enc.substring(0, sB64Enc.length - 2 + nMod3) +
    (nMod3 === 2 ? "" : nMod3 === 1 ? "=" : "==")
  );
}

const seen_states:Record<string,Uint8Array> = {};
const seen_saves:Record<string,null> = {};
function checkChangedStatesAndSaves() {
  const states = FS.readdir(state_dir);
  for (let state of states) {
    if(state == "." || state == "..") { continue; }
    const png_file = state.endsWith(".png") ? state : state + ".png";
    const state_file = state.endsWith(".png") ? state.substring(0,state.length-4) : state;
    if(!(state_file in seen_states) && file_exists(state_dir+"/"+png_file) && file_exists(state_dir+"/"+state_file)) {
      const img_data = FS.readFile(state_dir+"/"+png_file);
      seen_states[state_file] = img_data;
      const img_data_b64 = base64EncArr(img_data);
      ui_state.newState(state_file, img_data_b64);
    }
  }
  const saves = FS.readdir(saves_dir);
  for (let save of saves) {
    if(save == "." || save == "..") { continue; }
    if(!(save in seen_saves)) {
      seen_saves[save] = null;
      ui_state.newSave(save);
    }
  }
}

function file_exists(path:string) : boolean {
  return FS.analyzePath(path).exists
}

/** per https://github.com/eligrey/FileSaver.js/issues/774#issue-1393525742
  * @param {Blob} blob
  * @param {string} name
  */
function saveAs(blob:Blob, name:string) {
    // Namespace is used to prevent conflict w/ Chrome Poper Blocker extension (Issue https://github.com/eligrey/FileSaver.js/issues/561)
  const a = <HTMLAnchorElement>document.createElementNS('http://www.w3.org/1999/xhtml', 'a');
  a.download = name;
  a.rel = 'noopener';
  a.href = URL.createObjectURL(blob);

  setTimeout(() => URL.revokeObjectURL(a.href), 40 /* sec */ * 1000);
  setTimeout(() => a.click(), 0);
}
