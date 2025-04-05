import STYLES from './style.css?inline';
import * as ra from './ra';
import * as v86 from './v86';
import {EmuControls,EmbedOptions,ControllerOverlayMode} from './types.d';
import imgUrl from './canvas.svg';

// TODO replace with a shadow DOM thing?
let which_canvas = 0;

export async function embed(gisst:string, container:HTMLDivElement, options?:EmbedOptions) {
  if(which_canvas == 0) {
    const style = document.createElement("style");
    style.textContent = STYLES;
    document.head.appendChild(style);
  }
  
  container.classList.add("gisst-embed-webplayer-container");
  const canvas = document.createElement("canvas");
  canvas.tabIndex = 1; // make canvas focusable
  canvas.classList.add("gisst-embed-webplayer");
  canvas.classList.add("gisst-embed-hidden");
  canvas.addEventListener("contextmenu", (e) => e.preventDefault());
  canvas.id = "embed_canvas_"+which_canvas;
  which_canvas += 1;
  const canvas_txt = document.createElement("div");
  canvas_txt.classList.add("gisst-embed-webplayer-textmode");
  canvas_txt.classList.add("gisst-embed-hidden");
  const preview_img = document.createElement("img");
  preview_img.classList.add("gisst-embed-webplayer-preview");
  preview_img.src = imgUrl;
  preview_img.alt = "Loading Icon";
  const mute_a = document.createElement("a");
  mute_a.classList.add("gisst-embed-webplayer-mute");
  mute_a.classList.add("gisst-embed-webplayer-button");
  mute_a.text = "🔇";
  const halt_a = document.createElement("a");
  halt_a.classList.add("gisst-embed-webplayer-halt");
  halt_a.classList.add("gisst-embed-webplayer-button");
  halt_a.text = "❌";

  container.appendChild(canvas);
  container.appendChild(canvas_txt);
  container.appendChild(preview_img);

  // capture groups: root, UUID, query params
  const gisst_proto = gisst.slice(0,gisst.indexOf(":"));
  const gisst_http_proto = gisst_proto == "gisst" ? "https" : gisst_proto;
  gisst = gisst.replace("/play/", "/").replace("/data/", "/").replace("http:", "gisst:").replace("https:", "gisst:");
  const matches = gisst.match(/gisst:\/\/(.*)\/([0-9a-fA-F-]{32,})(\?.+)?$/);
  if(!matches) { throw "malformed gisst url"; }
  const gisst_root = matches[1];
  const gisst_query = matches[2] + (matches[3] || "");
  const data_resp = await fetch(gisst_http_proto+"://"+gisst_root+"/data/"+gisst_query, {headers:[["Accept","application/json"]]});
  console.log(data_resp);
  const config = await data_resp.json();
  console.log(config);
  const kind = config.environment.environment_framework;
  let emu:EmuControls;
  if(kind == "v86") {
    emu = await v86.init(gisst_http_proto+"://"+gisst_root, config.environment, config.start, config.manifest, container, options ?? {controls:ControllerOverlayMode.Auto});
  } else {
    emu = await ra.init(gisst_http_proto+"://"+gisst_root, config.environment.environment_core_name, config.start, config.manifest, container, options ?? {controls:ControllerOverlayMode.Auto});
  }
  container.appendChild(mute_a);
  container.appendChild(halt_a);
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
      canvas.remove();
      canvas_txt.remove();
      preview_img.remove();
      mute_a.remove();
      halt_a.remove();
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
    embed(src!, div, {controls:controller_mode_from(this.getAttribute("controller"))});
  }
}

customElements.define("gisst-embed", GISSTElement);
