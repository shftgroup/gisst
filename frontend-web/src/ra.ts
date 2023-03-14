import './style.css';
import IMG_STATE_ENTRY from './media/init_state.png';
import * as fetchfs from './fetchfs';
import {UI} from 'gisst-player';
import {saveAs,base64EncArr} from './util';
const FS_CHECK_INTERVAL = 1000;

const state_dir = "/home/web_user/retroarch/userdata/states";
const saves_dir = "/home/web_user/retroarch/userdata/saves";

const retro_args = ["-v"];

let ui_state:UI;

export function init(core:string, content_folder:string, content:string, entryState:boolean, movie:boolean) {
  let content_base = content.substring(0, content.lastIndexOf("."));
  if (entryState) {
    retro_args.push("-e");
    retro_args.push("1");
  }
  if (movie) {
    retro_args.push("-P");
    retro_args.push(state_dir+"/"+content_base+".replay0");
  } else {
    retro_args.push("-R");
    retro_args.push(state_dir+"/"+content_base+".replay0");
  }
  retro_args.push("--appendconfig");
  retro_args.push("/home/web_user/content/retroarch.cfg");
  retro_args.push("/home/web_user/content/" + content);
  console.log(retro_args);

  loadRetroArch(core,
    function () {
      let p1 = fetchfs.registerFetchFS(("assets/frontend/bundle/.index-xhr"), "assets/frontend/bundle", "/home/web_user/retroarch/bundle", true);
      let xfs_content_files: fetchfs.Index = { "retroarch.cfg": null };
      xfs_content_files[content] = null;
      if (entryState) {
        xfs_content_files["entry_state"] = null;
      }
      if (movie) {
        xfs_content_files["replay.replay"] = null;
      }
      let p2 = fetchfs.registerFetchFS(xfs_content_files, content_folder, "/home/web_user/content", false);
      let p3 = fetchfs.registerFetchFS({"retroarch_web_base.cfg":null}, "assets", "/home/web_user/retroarch/", false);
      fetchfs.mkdirp(saves_dir);
      fetchfs.mkdirp(state_dir);
      Promise.all([p1, p2, p3]).then(function () {
        copyFile("/home/web_user/retroarch/retroarch_web_base.cfg", "/home/web_user/retroarch/userdata/retroarch.cfg");
        // TODO if movie, it would be very cool to have a screenshot of the movie's init state copied in here
        if (entryState) {
          copyFile("/home/web_user/content/entry_state",
            state_dir + "/" + content_base + ".state1.entry");
          copyFile("/home/web_user/content/entry_state",
            state_dir + "/" + content_base + ".state1");
          if(file_exists("/home/web_user/content/entry_state.png")) {
            copyFile("/home/web_user/content/entry_state.png", state_dir+"/"+content_base+".state1.png");
          } else {
            fetch(IMG_STATE_ENTRY).then((resp) => resp.arrayBuffer()).then((buf) => FS.writeFile(state_dir+"/"+content_base+".state1.png", new Uint8Array(buf)));
          }
        }
        if (movie) {
          copyFile("/home/web_user/content/replay.replay0",
            state_dir + "/" + content_base + ".replay0");
          // if(file_exists("/home/web_user/content/entry_state.png")) {
          //   copyFile("/home/web_user/content/entry_state.png", state_dir+"/"+content_base+".state1.png");
          // } else {
          //   fetch(IMG_STATE_ENTRY).then((resp) => resp.arrayBuffer()).then((buf) => FS.writeFile(state_dir+"/"+content_base+".state1.png", new Uint8Array(buf)));
          // }
        } else {
          const f = FS.open(state_dir+"/"+content_base+".replay0", 'w');
          const te = new TextEncoder();
          FS.write(f, te.encode("\0"), 0, 1);
          FS.close(f);
        }
        retroReady();
      });
    });
}

function copyFile(from: string, to: string): void {
  let buf = FS.readFile(from);
  FS.writeFile(to, buf);
}

// TODO add clear button to call ui_state.clear()
function retroReady(): void {
  ui_state = new UI(
    <HTMLDivElement>document.getElementById("states")!,
    <HTMLDivElement>document.getElementById("replays")!,
    <HTMLDivElement>document.getElementById("saves")!,
    {
      "load_state":(num:number) => load_state_slot(num),
      "play_replay":(num:number) => play_replay_slot(num),
      "download_file":(category:"state" | "save" | "replay", file_name:string) => {
        let path = "/home/web_user/retroarch/userdata";
        if(category == "state") {
          path += "/states";
        } else if(category == "save") {
          path += "/saves";
        } else if(category == "replay") {
          path += "/states";
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
  retroArchSend("LOAD_STATE_SLOT "+n.toString());
}
function play_replay_slot(n:number) {
  retroArchSend("PLAY_REPLAY_SLOT "+n.toString());
}

const seen_states:Record<string,Uint8Array> = {};
const seen_saves:Record<string,null> = {};
const seen_replays:Record<string,null> = {};
function checkChangedStatesAndSaves() {
  const states = FS.readdir(state_dir);
  for (let state of states) {
    if(state == "." || state == "..") { continue; }
    if(state.endsWith(".png") || state.includes(".state")) {
      const png_file = state.endsWith(".png") ? state : state + ".png";
      const state_file = state.endsWith(".png") ? state.substring(0,state.length-4) : state;
      if(!(state_file in seen_states) && file_exists(state_dir+"/"+png_file) && file_exists(state_dir+"/"+state_file)) {
        const img_data = FS.readFile(state_dir+"/"+png_file);
        seen_states[state_file] = img_data;
        const img_data_b64 = base64EncArr(img_data);
        ui_state.newState(state_file, img_data_b64);
      }
    } else if(state.includes(".replay")) {
      if(!(state in seen_replays)) {
        seen_replays[state] = null;
        ui_state.newReplay(state);
      }
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
