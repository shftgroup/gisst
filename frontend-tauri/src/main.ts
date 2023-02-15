import { invoke } from "@tauri-apps/api/tauri";
import {copyFile} from "@tauri-apps/api/fs";
import {appCacheDir,join} from "@tauri-apps/api/path";
import { Command } from '@tauri-apps/api/shell';

async function run(core:string, content:string, entryState:bool, movie:bool) {
  console.assert(!(entryState && movie), "It is invalid to have both an entry state and play back a movie");
  let content_base = content.substring(0, content.lastIndexOf("."));
  let cache_dir = await appCacheDir();
  let content_dir = await join(cache_dir, "content");
  let retro_args = ["-v", "-c", await join(cache_dir, "retroarch.cfg"), "--appendconfig", await join(content_dir, "retroarch.cfg"), "-L", core];
  if (entryState) {
    retro_args.push("-e");
    retro_args.push("1");
  }
  if (movie) {
    retro_args.push("-P");
    retro_args.push(await join(content_dir,"/movie.bsv"));
  } else {
    retro_args.push("-R");
    retro_args.push(await join(content_dir, "/movie.bsv"));
  }
  retro_args.push(await join(content_dir, content));
  console.log(retro_args);
  if (entryState) {
    copyFile(await join(content_dir, "entry_state"), await join(cache_dir, "states", content_base+".state1.entry"));
  }

  const command = Command.sidecar('binaries/retroarch', retro_args);
  // TODO: watch proc, send commands, etc
  const proc = await command.execute();
  console.log("proc",proc.stdout,proc.stderr);
}

window.addEventListener("DOMContentLoaded", () => {
  document
    .querySelector("#run-cold-button")
    ?.addEventListener("click", () => run("fceumm", "bfight.nes", false, false));
  document
    .querySelector("#run-entry-button")
    ?.addEventListener("click", () => run("fceumm", "bfight.nes", true, false));
  document
    .querySelector("#run-movie-button")
    ?.addEventListener("click", () => run("fceumm", "bfight.nes", false, true));
});
