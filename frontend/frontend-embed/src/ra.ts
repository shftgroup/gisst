import {ColdStart, StateStart, ReplayStart, ObjectLink, EmuControls, EmbedOptions, ControllerOverlayMode} from './types.d';
import {loadRetroArch,LibretroModule} from './libretro_adapter';

export async function init(gisst_root:string, core:string, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[], container:HTMLDivElement, embed_options:EmbedOptions):Promise<EmuControls> {
  const state_dir = "/mem/states";
  const retro_args = ["-v"];
  const content = manifest.find((o) => o.object_role=="content" && o.object_role_index == 0)!;
  const content_file = content.file_filename!;
  const content_base = content_file.substring(0, content_file.lastIndexOf("."));
  const entryState = start.type == "state";
  const movie = start.type == "replay";
  const source_path = content.file_source_path!.replace(content.file_filename!, "");
  const use_gamepad_overlay = embed_options.controls == ControllerOverlayMode.On || ((embed_options.controls??ControllerOverlayMode.Auto) == ControllerOverlayMode.Auto && mobileAndTabletCheck());
  /* These two could be loaded concurrently using Promises.all */
  let state_data:Uint8Array|null = null;
  if (entryState) {
    retro_args.push("-e");
    retro_args.push("1");
    const data = (start as StateStart).data;
    state_data = new Uint8Array(await (await fetch(gisst_root+"/storage/"+data.file_dest_path)).arrayBuffer());
  }
  let replay:Uint8Array|null = null;
  if (movie) {
    retro_args.push("-P");
    retro_args.push(state_dir+"/"+content_base+".replay1");
    const data = (start as ReplayStart).data;
    replay = new Uint8Array(await ((await fetch(gisst_root+"/storage/"+data.file_dest_path)).arrayBuffer()));
  }
  retro_args.push("--config=/mem/retroarch.cfg");
  const has_config = manifest.find((o) => o.object_role=="config")!;
  if(has_config) {
    retro_args.push("--appendconfig");
    retro_args.push("/fetch/content/retroarch.cfg");
  }
  retro_args.push("/fetch/content/" + source_path + "/" + content.file_filename!);
  console.log(retro_args);
  let ra_cfg_text:string = await ((await fetch(gisst_root+"/assets/retroarch_web_base.cfg")).text());
  return new Promise((res) => {
    loadRetroArch(gisst_root, core, {'OPFS':'/home/web_user/retroarch', 'FETCH_MANIFEST':'/mem/fetch.txt'},
      true,
      async function (module:LibretroModule) {
        const enc = new TextEncoder();
        module.FS.createPath("/", "fetch/content", true, true);
        module.FS.createPath("/", state_dir, true, true);
        let fetch_manifest = "";
        /* TODO many of these awaits could be instead done simultaneously with Promise.all() */
        for(const file of manifest) {
          let download_source_path_full = "/fetch/content/" + file.file_source_path;
          let download_source_path = download_source_path_full;
          const last_index = download_source_path.lastIndexOf(file.file_filename!);
          if(last_index >= 0) {
            download_source_path = download_source_path_full.substring(0, last_index);
          } else {
            download_source_path_full += `/${file.file_filename!}`;
          }
          const content_url = gisst_root+"/storage/"+file.file_dest_path;
          const resp = await fetch(content_url, {method:"HEAD"});
          let sz = 0;
          if (resp.status == 200) {
            sz = parseInt(resp.headers.get("Content-Length") ?? "0",10);
          }
          await resp.text();
          if (sz > 0 && sz <= 16*1024*1024) {
            const data = await (await fetch(content_url)).arrayBuffer();
            module.FS.createPath("/",download_source_path, true, true);
            module.FS.createDataFile(download_source_path, file.file_filename!, new Uint8Array(data), true, true, true);
          } else {
            const content_url_encoded = encodeURI(content_url);
            fetch_manifest += `${content_url_encoded} ${download_source_path_full}\n`;
          }
        }
        module.FS.createDataFile("/mem", "fetch.txt", enc.encode(fetch_manifest), true, true, true);
        if (entryState) {
          module.FS.createDataFile(state_dir, content_base + ".state1.entry", state_data, true, true, true);
          module.FS.createDataFile(state_dir, content_base + ".state1", state_data, true, true, true);
        }
        if (movie) {
          module.FS.createPath("/", state_dir, true, true);
          module.FS.createDataFile(state_dir, content_base + ".replay1", replay, true, true, true);
        }
        if (use_gamepad_overlay) {
          // gameboy, gba, nes, snes, retropad
          // gambatte, vba_next, fceumm, snes9x
          const overlays = {
            "gambatte": "gameboy",
            "vba_next": "gba",
            "fceumm": "nes",
            "snes9x": "snes",
          };
          let overlay = "retropad";
          if (core in overlays) {
            overlay = overlays[core as keyof typeof overlays];
          }
          ra_cfg_text += "\ninput_overlay_enable = \"true\"\ninput_overlay = \"/home/web_user/retroarch/bundle/overlays/gamepads/"+overlay+"/"+overlay+".cfg\"\ninput_overlay_enable_autopreferred = \"true\"";
        }
        const lines_enc = enc.encode(ra_cfg_text);
        module.FS.createDataFile("/mem", "retroarch.cfg", lines_enc, true, true, true);
        retroReady(module, retro_args, container);
        res({
          toggle_mute:() => {
            send_message(module, "MUTE");
          },
          halt:async () => {
            await send_message(module, "QUIT");
            await sleep(50);
            await send_message(module, "QUIT");
          }
        })
      });
  });
}

function sleep(ms:number) : Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function retroReady(module:LibretroModule, retro_args:string[], container:HTMLDivElement): void {
  const preview = container.getElementsByTagName("img")[0];
  preview.classList.add("gisst-embed-loaded");
  preview.addEventListener(
    "click",
    function () {
      const canv = <HTMLCanvasElement>container.getElementsByTagName("canvas")[0]!;
      preview.classList.add("gisst-embed-hidden");
      module.startRetroArch(canv, retro_args, function () {
        canv.classList.remove("gisst-embed-hidden");
      });
      return false;
    });
}

async function read_response(module:LibretroModule, wait:boolean): Promise<string | undefined> {
  const waiting:() => Promise<string|undefined> = () => new Promise((resolve) => {
    /* eslint-disable prefer-const */
    let interval:ReturnType<typeof setInterval>;
    const read_cb = () => {
      const resp = module.retroArchRecv();
      if(resp != undefined) {
        clearInterval(interval!);
        resolve(resp);
      }
    }
    interval = setInterval(read_cb, 100);
  });
  let outp:string|undefined=undefined;
  if(wait) {
    outp = await waiting();
  } else {
    outp = module.retroArchRecv();
  }
  // console.log("stdout: ",outp);
  return outp;
}

async function send_message(module:LibretroModule, msg:string) {
  let clearout = await read_response(module, false);
  while(clearout) { clearout = await read_response(module, false); }
  // console.log("send:",msg);
  module.retroArchSend(msg+"\n");
}

// TYVM https://stackoverflow.com/a/11381730
function mobileAndTabletCheck() {
  let check = false;
  (function(a){if(/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino|android|ipad|playbook|silk/i.test(a)||/1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw-(n|u)|c55\/|capi|ccwa|cdm-|cell|chtm|cldc|cmd-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc-s|devi|dica|dmob|do(c|p)o|ds(12|-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(-|_)|g1 u|g560|gene|gf-5|g-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd-(m|p|t)|hei-|hi(pt|ta)|hp( i|ip)|hs-c|ht(c(-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i-(20|go|ma)|i230|iac( |-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|-[a-w])|libw|lynx|m1-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|-([1-8]|c))|phil|pire|pl(ay|uc)|pn-2|po(ck|rt|se)|prox|psio|pt-g|qa-a|qc(07|12|21|32|60|-[2-7]|i-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h-|oo|p-)|sdk\/|se(c(-|0|1)|47|mc|nd|ri)|sgh-|shar|sie(-|m)|sk-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h-|v-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl-|tdg-|tel(i|m)|tim-|t-mo|to(pl|sh)|ts(70|m-|m3|m5)|tx-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas-|your|zeto|zte-/i.test(a.substr(0,4))) check = true;})(navigator.userAgent||navigator.vendor);
  return check;
}

