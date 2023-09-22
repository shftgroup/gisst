import './style.css';
import * as ra from './ra';
import * as v86 from './v86';
import imgUrl from './canvas.png';

export async function embed(gisst:string, container:HTMLDivElement) {
  container.classList.add("gisst-embed-webplayer-container");
  const canvas = document.createElement("canvas");
  canvas.tabIndex = 1; // make canvas focusable
  canvas.classList.add("gisst-embed-webplayer");
  canvas.classList.add("gisst-embed-hidden");
  canvas.addEventListener("contextmenu", (e) => e.preventDefault());
  const canvas_txt = document.createElement("div");
  canvas_txt.classList.add("gisst-embed-webplayer-textmode");
  canvas_txt.classList.add("gisst-embed-hidden");
  const preview_img = document.createElement("img");
  preview_img.classList.add("gisst-embed-webplayer-preview");
  preview_img.src = imgUrl;
  preview_img.width = 960;
  preview_img.height = 720;
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
  container.appendChild(mute_a);
  container.appendChild(halt_a);

  // capture groups: root, UUID, query params
  const gisst_proto = gisst.slice(0,gisst.indexOf(":"));
  const gisst_http_proto = gisst_proto == "gisst" ? "https" : gisst_proto;
  gisst = gisst.replace("/play/", "/").replace("http:", "gisst:").replace("https:", "gisst:");
  const matches = gisst.match(/gisst:\/\/(.*)\/([0-9a-fA-F-]{32,})(\?.+)?$/);
  if(!matches) { throw "malformed gisst url"; }
  const gisst_root = matches[1];
  const gisst_query = matches[2] + (matches[3] || "");
  const data_resp = await fetch(gisst_http_proto+"://"+gisst_root+"/play/"+gisst_query, {headers:[["Accept","application/json"]]});
  console.log(data_resp);
  const config = await data_resp.json();
  console.log(config);
  const kind = config.environment.environment_framework;
  if(kind == "v86") {
    v86.init(gisst_http_proto+"://"+gisst_root, config.environment, config.start, config.manifest, container);
  } else {
    ra.init(gisst_http_proto+"://"+gisst_root, config.environment.environment_core_name, config.start, config.manifest, container);
  }
}
