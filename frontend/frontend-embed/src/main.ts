import './style.css';
import * as ra from './ra';
import * as v86 from './v86';
import {EmuControls,EmbedOptions,ControllerOverlayMode} from './types.d';
import imgUrl from './canvas.svg';

let which_canvas = 0;
//TODO: UI testing to make sure the code is still doing the same thing
async function createContainerUI(container:HTMLDivElement) {
  const element_string = `
    <canvas class="gisst-embed-webplayer gisst-embed-hidden" tabindex="1" id="embed_canvas_${which_canvas}"></canvas>
    <div class="gisst-embed-webplayer-textmode gisst-embed-hidden"></div>
    <img class="gisst-embed-webplayer-perview" src="${imgUrl}" alt="Loading Icon"></img>
    <a class="gisst-embed-webplayer-mute gisst-embed-webplayer-button gisst-embed-hidden" >🔇</a>
    <a class="gisst-embed-webplayer-halt gisst-embed-webplayer-button gisst-embed-hidden" >❌</a>
  `;

  container.innerHTML = element_string;

  container.querySelector(`#embed_canvas_${which_canvas}`)!.addEventListener("contextmenu", e => e.preventDefault())

  which_canvas += 1;
}

export async function embed(gisst:string, container:HTMLDivElement, options?:EmbedOptions) {
  createContainerUI(container)

  const mute_a = container.querySelector("a.gisst-embed-webplayer-mute")! as HTMLLinkElement;
  const halt_a = container.querySelector("a.gisst-embed-webplayer-halt")! as HTMLLinkElement;
  const canvas = container.querySelector("canvas.gisst-embed-webplayer")! as HTMLCanvasElement;

  const config = await fetchConfig(gisst);
  if (!config) return

  const kind = config.environment.environment_framework;

  let emu:EmuControls;
  const [gisst_http_proto, gisst_root, _] = parseGisstUrl(gisst)
  if(kind == "v86") {
    emu = await v86.init(gisst_http_proto+"://"+gisst_root, config.environment, config.start, config.manifest, container, options ?? {controls:ControllerOverlayMode.Auto});
  } else {
    emu = await ra.init(gisst_http_proto+"://"+gisst_root, config.environment.environment_core_name, config.start, config.manifest, container, options ?? {controls:ControllerOverlayMode.Auto});
  }

  mute_a.classList.remove("gisst-embed-hidden");
  halt_a.classList.remove("gisst-embed-hidden");

  mute_a.addEventListener(
    "click",
    function () {
      emu.toggle_mute();
    }
  );
  halt_a.addEventListener(
    "click",
    async function () {
      await emu.halt();
      container.innerHTML = "";
    }
  );

  const ro = new ResizeObserver((_entries, _observer) => {
    const w = canvas.width;
    const h = canvas.height;
    if (w == 0 || h == 0) { return; }
    const target_w = container.offsetWidth;
    let target_h;
    if (kind == "v86") {
      const aspect = w / h;
      target_h = target_w / aspect;
    } else {
      target_h = container.offsetHeight;
    }
    const new_w = `${target_w}px`;
    const new_h = `${target_h}px`;
    if (canvas.style.width != new_w || canvas.style.height != new_h) {
      canvas.style.width = new_w;
      canvas.style.height = new_h;
    }
  })
  ro.observe(canvas);
  ro.observe(container);
  canvas.style.touchAction = "none";
  canvas.addEventListener("touchstart", touchHandler, true);
  canvas.addEventListener("touchmove", touchHandler, true);
  canvas.addEventListener("touchend", touchHandler, true);
  canvas.addEventListener("touchcancel", touchHandler, true);
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

export async function fetchConfig(gisst: string) {
  // capture groups: root, UUID, query params
  const [gisst_http_proto, gisst_root, gisst_query] = parseGisstUrl(gisst)

  const data_resp = await fetch(gisst_http_proto+"://"+gisst_root+"/data/"+gisst_query, {headers:[["Accept","application/json"]]});
  console.log(data_resp);

  if(data_resp.status == 500) {
    throw new Error("Request Status 500: Internal Server Error")
  } else if (data_resp.status == 404) {
    throw new Error("Request Status 404: Instance Not Found")
  }

  const config = await data_resp.json();
  console.log(config);

  return config;
}

export function parseGisstUrl(gisst:string): [http_proto: string, root: string, query: string] {
  const proto = gisst.slice(0,gisst.indexOf(":"));

  const formated = gisst.replace("/play/", "/").replace("/data/", "/").replace("http:", "gisst:").replace("https:", "gisst:");
  const matches = formated.match(/gisst:\/\/(.*)\/([0-9a-fA-F-]{32,})(\?.+)?$/);

  if(!matches) { throw "malformed gisst url"; }

  return [proto == "gisst" ? "https" : proto, matches[1], matches[2] + (matches[3] || "")]
}
