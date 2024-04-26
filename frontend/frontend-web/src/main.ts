import 'gisst-player/style.css';
import * as ra from './ra';
import * as v86 from './v86';

window.onload = function() {
  const config = JSON.parse(document.getElementById("config")!.textContent!);
  const kind = config.environment.environment_framework;
  let storage = await navigator.storage.getDirectory();
  storage = await storage.getDirectoryHandle(`${config.instance.instance_id}_${config.environment.environment_id}`, {create:true});
  let db = new GISSTDBConnector(`${window.location.protocol}//${window.location.host}`);
  if(kind == "v86") {
    v86.init(storage, db, config.environment, config.start, config.manifest, config.boot_into_record);
  } else {
    ra.init(storage, db, config.environment.environment_core_name, config.start, config.manifest, config.boot_into_record);
  }
};
