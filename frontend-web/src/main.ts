import './style.css'
import * as ra from './ra';
import * as v86 from './v86';

/*

    {"environment":{"created_on":null,"environment_config":{"bios":{"url":"seabios.bin"},"fda":{"async":true,"url":"$CONTENT"},"vgabios":{"url":"vgabios.bin"}},"environment_core_name":"v86","environment_core_version":"0.1.0","environment_derived_from":null,"environment_framework":"v86","environment_id":"00000000-0000-0000-0000-000000000001","environment_name":"FreeDOS"},"instance":{"created_on":null,"environment_id":"00000000-0000-0000-0000-000000000001","instance_config":null,"instance_id":"00000000-0000-0000-0000-000000000001","work_id":"00000000-0000-0000-0000-000000000001"},"manifest":[{"object_dest_path":"0/0/0/0","object_filename":"freedos722.img","object_hash":"07cd9656778aa01e7f99f37e31b76e24","object_id":"00000000-0000-0000-0000-000000000002","object_role":"content","object_source_path":"freedos722.img"}],"save":null,"start":{"type":"cold"}}
        
    
 */
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
