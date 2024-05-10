import 'gisst-player/style.css';
import * as ra from './ra';
import * as v86 from './v86';

window.onload = async function() {
  const config = JSON.parse(document.getElementById("config")!.textContent!);
  const kind = config.environment.environment_framework;
  if(kind == "v86") {
    await v86.init(config.environment, config.start, config.manifest, config.boot_into_record);
  } else {
    await ra.init(config.environment.environment_core_name, config.start, config.manifest, config.boot_into_record);
  }
  const container = <HTMLCanvasElement>document.getElementById("canvas_div")!;
  const canv = <HTMLCanvasElement>document.getElementById("canvas")!;
  const ro = new ResizeObserver((_entries, _observer) => {
    const w = canv.width;
    const h = canv.height;
    if (w == 0 || h == 0) { return; }
    const aspect = w/h;
    const target_w = container.offsetWidth;
    const target_h = target_w / aspect;
    if (w > target_w) {
      canv.style.width = `${target_w}px`;
      canv.style.height = `${target_h}px`; // h/w * w = h
    }
  })
  ro.observe(canv);
  ro.observe(container);
};
