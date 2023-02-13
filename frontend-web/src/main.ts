import './style.css'
import * as fetch from './fetchfs'

const core = "fceumm";
const content_folder = "/content/";
const content = "bfight.nes";
const entryState = true;
const movie = false;

const FS_CHECK_INTERVAL = 1000;

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
    Promise.all([p1, p2]).then(function () {
      if (entryState) {
        fetch.mkdirp("/home/web_user/retroarch/userdata/states");
        copyFile("/home/web_user/content/entry_state",
          "/home/web_user/retroarch/userdata/states/" + content_base + ".state1.entry");
      }
      copyFile("/home/web_user/content/retroarch.cfg", "/home/web_user/retroarch/userdata/retroarch.cfg");
      retroReady();
    });
  });

function copyFile(from: string, to: string): void {
  let buf = FS.readFile(from);
  FS.writeFile(to, buf);
}

function retroReady(): void {
  let prev = document.getElementById("webplayer-preview")!;
  prev.classList.add("loaded");
  prev.addEventListener(
    "click",
    function () {
      let canv = <HTMLCanvasElement>document.getElementById("canvas")!;
      prev.classList.add("hidden");
      startRetroArch(canv, retro_args, function () {
        let canv = document.getElementById("canvas")!;
        setInterval(checkChangedSaves, FS_CHECK_INTERVAL);
        canv.classList.remove("hidden");
      });
      return false;
    });
}

var renderedSaves = [];
function checkChangedSaves() {
  try {
    var newSaves = FS.readdir("/home/web_user/retroarch/userdata/states");
    // if any new ones, update lastSaves
    for (var i = renderedSaves.length; i < newSaves.length; i++) {
      let save = newSaves[i];
      console.log(save);
      renderedSaves.push(save);
    }
  } catch (e) {
    // if (e instanceof ErrnoError) {
    //     // do nothing
    // } else {
    //     throw e;
    // }
  }
}
