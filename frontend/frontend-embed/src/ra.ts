import * as fetchfs from './fetchfs';
import {ColdStart, StateStart, ReplayStart, ObjectLink, EmuControls} from './types';
import {loadRetroArch,LibretroModule} from './libretro_adapter';

export async function init(gisst_root:string, core:string, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[], container:HTMLDivElement):Promise<EmuControls> {
  const state_dir = "/home/web_user/retroarch/userdata/states";
  const saves_dir = "/home/web_user/retroarch/userdata/saves";
  const retro_args = ["-v"];
  const content = manifest.find((o) => o.object_role=="content")!;
  const content_file = content.file_filename!;
  const dash_point = content_file.indexOf("-");
  const content_base = content_file.substring(dash_point < 0 ? 0 : dash_point, content_file.lastIndexOf("."));
  const entryState = start.type == "state";
  const movie = start.type == "replay";
  if (entryState) {
    retro_args.push("-e");
    retro_args.push("1");
  }
  if (movie) {
    retro_args.push("-P");
    retro_args.push(state_dir+"/"+content_base+".replay1");
  }
  retro_args.push("--appendconfig");
  retro_args.push("/home/web_user/content/retroarch.cfg");
  retro_args.push("/home/web_user/content/" + content_file);
  console.log(retro_args);
  return new Promise((res) => {
    loadRetroArch(gisst_root, core,
      function (module:LibretroModule) {
        fetchfs.mkdirp(module,"/home/web_user/content");

      const proms = [];

      proms.push(fetchfs.fetchZip(module,gisst_root+"/assets/frontend/bundle.zip","/home/web_user/retroarch/"));

      for(const file of manifest) {
        const file_prom = fetchfs.fetchFile(module,gisst_root+"/storage/"+file.file_dest_path+"/"+file.file_hash+"-"+file.file_filename,"/home/web_user/content/"+file.file_source_path);
        proms.push(file_prom);
      }
      if (entryState) {
        // Cast: This one is definitely a statestart because the type is state
        const data = (start as StateStart).data;
        console.log(data, "/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/entry_state");
        proms.push(fetchfs.fetchFile(module,gisst_root+"/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/entry_state"));
      }
      if (movie) {
        // Cast: This one is definitely a replaystart because the type is state
        const data = (start as ReplayStart).data;
        console.log(data, "/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/replay.replay1");
        proms.push(fetchfs.fetchFile(module,gisst_root+"/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/replay.replay1"));
      }
      proms.push(fetchfs.registerFetchFS(module,{"retroarch_web_base.cfg":null}, gisst_root+"/assets", "/home/web_user/retroarch/"));
      fetchfs.mkdirp(module,saves_dir);
      fetchfs.mkdirp(module,state_dir);
      Promise.all(proms).then(function () {
        copyFile(module,"/home/web_user/retroarch/retroarch_web_base.cfg", "/home/web_user/retroarch/userdata/retroarch.cfg");
        // TODO if movie, it would be very cool to have a screenshot of the movie's init state copied in here
        if (entryState) {
          copyFile(module,"/home/web_user/content/entry_state",
            state_dir + "/" + content_base + ".state1.entry");
          copyFile(module,"/home/web_user/content/entry_state",
            state_dir + "/" + content_base + ".state1");
        }
        if (movie) {
          console.log("Put movie in",state_dir + "/" + content_base + ".replay1");
          copyFile(module,"/home/web_user/content/replay.replay1", state_dir + "/" + content_base + ".replay1");
        } else {
          const f = module.FS.open(state_dir+"/"+content_base+".replay1", 'w');
          const te = new TextEncoder();
          module.FS.write(f, te.encode("\0"), 0, 1);
          module.FS.close(f);
        }
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
  });
}

function sleep(ms:number) : Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function copyFile(module:LibretroModule, from: string, to: string): void {
  const buf = module.FS.readFile(from);
  module.FS.writeFile(to, buf);
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
