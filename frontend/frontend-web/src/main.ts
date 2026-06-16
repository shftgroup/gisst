import {UI,GISSTModels} from 'gisst-player';
import 'gisst-player/style.css';
import * as ra from './ra';
import * as v86 from './v86';
import {embed} from 'frontend-embed';
import {ControllerOverlayMode} from 'frontend-embed/types';

window.onload = async function() {
  const gisst_url = import.meta.env.DEV ? document.getElementById("gisst_url")!.textContent : window.location.href;
  const params = new URLSearchParams(gisst_url.slice(gisst_url.indexOf("?")+1));
  const options = {
    controls:ControllerOverlayMode.Auto,
    record_from_start:params.get("boot_into_record"),
    record_video:true
  };
  const ui = new UI(
    <HTMLDivElement>document.getElementById("ui")!,
    false,
  );
  const embedded = await embed(gisst_url, ui.emulator_div, options);
  ui.setConfig(embedded as GISSTModels.FrontendConfig);
  if(embedded.environment.environment_framework == "v86") {
    await v86.init(ui, embedded);
  } else {
    await ra.init(ui, embedded);
  }
}
