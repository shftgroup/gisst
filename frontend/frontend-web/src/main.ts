import 'gisst-player/style.css';
import * as ra from './ra';
import * as v86 from './v86';
import {ControllerOverlayMode} from './types.d';

window.onload = async function() {
  const config = JSON.parse(document.getElementById("config")!.textContent!);
  const gisst_root = config.gisst_root;
  const kind = config.environment.environment_framework;
  if(kind == "v86") {
    await v86.init(gisst_root, config.environment, config.start, config.manifest, config.boot_into_record, config.embed_options??{controls:ControllerOverlayMode.Auto});
  } else {
    await ra.init(gisst_root, config.environment.environment_core_name, config.start, config.saves, config.manifest, config.boot_into_record, config.embed_options??{controls:ControllerOverlayMode.Auto});
  }
  const container = <HTMLCanvasElement>document.getElementById("canvas_div")!;
  const canv = <HTMLCanvasElement>document.getElementById("canvas")!;
  const ro = new ResizeObserver((_entries, _observer) => {
    const w = canv.width;
    const h = canv.height;
    if (w == 0 || h == 0) { return; }
    const target_w = container.offsetWidth;
    let target_h = container.offsetHeight;
    if (kind == "v86") {
      const aspect = w / h;
      target_h = target_w / aspect;
    }
    const new_w = `${target_w}px`;
    const new_h = `${target_h}px`;
    if (canv.style.width != new_w || canv.style.height != new_h) {
      canv.style.width = new_w;
      canv.style.height = new_h;
    }
  })
  ro.observe(canv);
  ro.observe(container);

  canv.style.touchAction = "none";
  canv.addEventListener("touchstart", touchHandler, true);
  canv.addEventListener("touchmove", touchHandler, true);
  canv.addEventListener("touchend", touchHandler, true);
  canv.addEventListener("touchcancel", touchHandler, true);
}
// qua https://stackoverflow.com/a/1781750
function touchHandler(event:TouchEvent)
{
    const touches = event.changedTouches, first = touches[0];
    let type = "";
    switch(event.type)
    {
        case "touchstart": type = "mousedown"; break;
        case "touchmove":  type = "mousemove"; break;
        case "touchend":   type = "mouseup";   break;
        default:           return;
    }
    const simulatedEvent = document.createEvent("MouseEvent");
    simulatedEvent.initMouseEvent(type, true, true, window, 1,
                                  first.screenX, first.screenY,
                                  first.clientX, first.clientY, false,
                                  false, false, false, 0/*left*/, null);
    first.target.dispatchEvent(simulatedEvent);
    event.preventDefault();
}
