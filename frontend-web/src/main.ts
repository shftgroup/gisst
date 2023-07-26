import './style.css'
import * as ra from './ra';
import * as v86 from './v86';

window.onload = function() {
  const config = JSON.parse(document.getElementById("config")!.textContent!);
  let kind = config.environment.environment_framework;
  if(kind == "v86") {
    (<HTMLImageElement>document.getElementById("webplayer-preview")!).src = "/media/canvas-v86.png";
    v86.init(config.environment, config.start, config.manifest);
  } else {
    (<HTMLImageElement>document.getElementById("webplayer-preview")!).src = "/media/canvas-ra.png";
    ra.init(config.environment.environment_core_name, config.start, config.manifest);
  }
};
