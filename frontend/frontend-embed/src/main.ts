import STYLES from './style.css?inline';
import * as ra from './ra';
import * as v86 from './v86';
import {EmuControls,EmbedOptions,ControllerOverlayMode} from './types.d';
import imgUrl from './canvas.svg';
export * as types from './types.d';

// TODO replace with a shadow DOM thing?
let which_canvas = 0;

async function createContainerUI(container: HTMLDivElement) {
  const element_string = `
    <canvas class="gisst-embed-webplayer gisst-embed-hidden" tabindex="1" id="embed_canvas_${which_canvas}"></canvas>
    <div class="gisst-embed-webplayer-textmode gisst-embed-hidden"></div>
    <img class="gisst-embed-webplayer-preview" src="${imgUrl}" alt="Loading Icon"></img>
    <a class="gisst-embed-webplayer-mute gisst-embed-webplayer-button gisst-embed-hidden" >🔇</a>
    <a class="gisst-embed-webplayer-halt gisst-embed-webplayer-button gisst-embed-hidden" >❌</a>
  `;

  container.innerHTML = element_string;

  container.querySelector(`#embed_canvas_${which_canvas}`)!.addEventListener("contextmenu", e => e.preventDefault())

  which_canvas += 1

}

export async function fetchConfig(gisst_http_proto: string, gisst_root: string, gisst_query: string) {
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

  const formatted = gisst.replace("/play/", "/").replace("/data/", "/").replace("http:", "gisst:").replace("https:", "gisst:");
  const matches = formatted.match(/gisst:\/\/(.*)\/([0-9a-fA-F-]{32,})(\?.+)?$/);

  const result_proto = proto == "gisst" ? "https" : proto;
  let result_host;
  let result_uuid;
  let result_startparams;
  if(matches) {
    result_host = matches[1];
    result_uuid = matches[2];
    result_startparams = matches[3] || "";
  } else {
    const same_origin_matches = formatted.match(/gisst:\/\/([0-9a-fA-F-]{32,})(\?.+)?$/);
    if (!same_origin_matches) {
      throw "malformed gisst url";
    }
    result_host = document.location.origin.slice(document.location.origin.indexOf("://")+3);
    result_uuid = same_origin_matches[1];
    result_startparams = same_origin_matches[2] || "";
  }

  return [result_proto, result_host, result_uuid + result_startparams]
}

export async function embed(gisst:string, container:HTMLDivElement, options?:EmbedOptions) : Promise<EmuControls> {
  if(which_canvas == 0) {
    const style = document.createElement("style");
    style.textContent = STYLES;
    document.head.appendChild(style);
  }
  createContainerUI(container);
  const mute_a = container.querySelector("a.gisst-embed-webplayer-mute")! as HTMLLinkElement;
  const halt_a = container.querySelector("a.gisst-embed-webplayer-halt")! as HTMLLinkElement;
  const canvas = container.querySelector("canvas.gisst-embed-webplayer")! as HTMLCanvasElement;
  canvas.style.width = container.style.width;
  canvas.style.height = container.style.height;
  const [gisst_http_proto, gisst_root, gisst_query] = parseGisstUrl(gisst);
  const config = await fetchConfig(gisst_http_proto, gisst_root, gisst_query);
  if (!config) throw "Failed to fetch config";
  const kind = config.environment.environment_framework;
  let emu:EmuControls;
  if(kind == "v86") {
    emu = await v86.init(gisst_http_proto+"://"+gisst_root, config.environment, config.work, config.instance, config.start, config.core_manifest, config.manifest, config.saves, container, options ?? {controls:ControllerOverlayMode.Auto, record_from_start:false});
  } else {
    emu = await ra.init(gisst_http_proto+"://"+gisst_root, config.environment, config.work, config.instance, config.start, config.saves, config.core_manifest, config.manifest, container, options ?? {controls:ControllerOverlayMode.Auto, record_from_start:false});
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
  canvas.style.touchAction = "none";
  canvas.addEventListener("touchstart", touchHandler, true);
  canvas.addEventListener("touchmove", touchHandler, true);
  canvas.addEventListener("touchend", touchHandler, true);
  canvas.addEventListener("touchcancel", touchHandler, true);
  return emu;
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


function controller_mode_from(s:string|null) : ControllerOverlayMode {
  if(s == "on") { return ControllerOverlayMode.On; }
  else if(s == "off") { return ControllerOverlayMode.Off; }
  else { return ControllerOverlayMode.Auto; }
}

class GISSTElement extends HTMLElement {
  static observedAttributes = ["src", "controller", "width", "height"];

  constructor() {
    super();
  }

  connectedCallback() {
    const src = this.getAttribute("src");
    if(!src) {
      throw "Cannot create GISST embed without src attribute";
    }
    const div = document.createElement("div");
    div.style.width = this.getAttribute("width") ?? "auto";
    div.style.height = this.getAttribute("height") ?? "auto";
    this.appendChild(div);
    embed(src!, div, {controls:controller_mode_from(this.getAttribute("controller")),record_from_start:false});
  }
}
customElements.define("gisst-embed", GISSTElement);
