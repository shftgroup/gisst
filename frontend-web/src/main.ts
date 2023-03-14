import './style.css'
import * as ra from './ra';
import * as v86 from './v86';

let core = "v86";
const content_folder = "/content/";
const content = "freedos722-root.json";
const entryState = false;
const movie = false;

window.onload = function() {
  console.assert(!(entryState && movie), "It is invalid to have both an entry state and play back a movie");
  if(core == "v86") {
    (<HTMLImageElement>document.getElementById("webplayer-preview")!).src = "media/canvas-v86.png";
    v86.init(content_folder, content, entryState, movie);
  } else {
    (<HTMLImageElement>document.getElementById("webplayer-preview")!).src = "media/canvas-ra.png";
    ra.init(core, content_folder, content, entryState, movie);
  }
};
