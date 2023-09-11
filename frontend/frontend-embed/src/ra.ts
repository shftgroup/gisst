import * as fetchfs from './fetchfs';
import {ColdStart, StateStart, ReplayStart, ObjectLink} from './types';


export function init(gisst_root:string, core:string, start:ColdStart | StateStart | ReplayStart, manifest:ObjectLink[], container:HTMLDivElement) {
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

  return loadRetroArch(core,
    function (module:LibretroModule) {
      fetchfs.mkdirp(module,"/home/web_user/content");

      const proms = [];

      proms.push(fetchfs.fetchZip(module,gisst_root+"/assets/frontend/bundle.zip","/home/web_user/retroarch/"));

      for(const file of manifest) {
        const file_prom = fetchfs.fetchFile(module,gisst_root+"/storage/"+file.file_dest_path+"/"+file.file_hash+"-"+file.file_filename,"/home/web_user/content/"+file.file_source_path,true);
        proms.push(file_prom);
      }
      if (entryState) {
        // Cast: This one is definitely a statestart because the type is state
        const data = (start as StateStart).data;
        console.log(data, "/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/entry_state");
        proms.push(fetchfs.fetchFile(module,gisst_root+"/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/entry_state",false));
      }
      if (movie) {
        // Cast: This one is definitely a replaystart because the type is state
        const data = (start as ReplayStart).data;
        console.log(data, "/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/replay.replay1");
        proms.push(fetchfs.fetchFile(module,gisst_root+"/storage/"+data.file_dest_path+"/"+data.file_hash+"-"+data.file_filename,"/home/web_user/content/replay.replay1",false));
      }
      proms.push(fetchfs.registerFetchFS(module,{"retroarch_web_base.cfg":null}, gisst_root+"/assets", "/home/web_user/retroarch/", false));
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
      });
    });
}

function copyFile(module:LibretroModule, from: string, to: string): void {
  const buf = module.FS.readFile(from);
  module.FS.writeFile(to, buf);
}

function retroReady(module:LibretroModule, retro_args:string[], preview:HTMLDivElement): void {
  preview.classList.add("gisst-embed-loaded");
  preview.addEventListener(
    "click",
    function () {
      const canv = <HTMLCanvasElement>preview.getElementsByTagName("canvas")[0]!;
      preview.classList.add("gisst-embed-hidden");
      module.startRetroArch(canv, retro_args, function () {
        canv.classList.remove("gisst-embed-hidden");
      });
      return false;
    });
}
