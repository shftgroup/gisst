import 'gisst-player/style.css';
import * as ra from './ra';
import * as v86 from './v86';

window.onload = function() {
  const config = JSON.parse(document.getElementById("config")!.textContent!);
  const kind = config.environment.environment_framework;
  if(kind == "v86") {
    v86.init(config.environment, config.start, config.manifest, config.boot_into_record);
  } else {
    ra.init(config.environment.environment_core_name, config.start, config.manifest, config.boot_into_record);
  }
};
