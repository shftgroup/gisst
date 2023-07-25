import './style.css'
import * as ra from './ra';
// import * as v86 from './v86';

/*
    {"environment":{"environment_core_name":"fceumm","environment_core_version":"1.52.0","created_on":null,"environment_config":null,"environment_derived_from":null,"environment_id":"00000000-0000-0000-0000-000000000000","environment_name":"Nintendo Entertainment System", "environment_framework":"retroarch"},"instance":{"created_on":null,"environment_id":"00000000-0000-0000-0000-000000000000","instance_config":null,"instance_id":"00000000-0000-0000-0000-000000000000","work_id":"00000000-0000-0000-0000-000000000000"},"manifest":[{"object_dest_path":"0/0/0/0","object_filename":"bfight.nes","object_hash":"6e125395ca4f18addb8ce6c9152dea85","object_id":"00000000-0000-0000-0000-000000000000","object_role":"content","object_source_path":"bfight.nes"},{"object_dest_path":"0/0/0/0","object_filename":"retroarch.cfg","object_hash":"a028e5747a6d0a658060644f56663d51","object_id":"00000000-0000-0000-0000-000000000001","object_role":"config","object_source_path":"retroarch.cfg"}],"save":null,"start":"Cold"}
    
    
 */
window.onload = function() {
  const config = JSON.parse(document.getElementById("config")!.textContent!);
  let kind = config.environment.environment_framework;
  if(kind == "v86") {
    // let content_folder = "/content/";
    // let _content = config.manifest.find((obj) => obj.object_role=="content");
    // let config_file = config.manifest.find((obj) => obj.object_role=="config");
    // // TODO scan config.start for these two
    // let entryState = false;
    // let movie = false;
    // (<HTMLImageElement>document.getElementById("webplayer-preview")!).src = "/media/canvas-v86.png";
    // // TODO: get this to work with images
    // v86.init(config.environment, config.start, config.manifest);
  } else {
    (<HTMLImageElement>document.getElementById("webplayer-preview")!).src = "/media/canvas-ra.png";
    ra.init(config.environment.environment_core_name, config.start, config.manifest);
  }
};
