// @ts-check
/// <reference lib="ESNext" />
/// <reference lib="webworker" />

export {};

import * as zip from "@zip.js/zip.js";

interface SyncAccessHandle {
  write(data:Uint8Array):number;
  close():void;
  flush():void;
  getSize():number;
  read(buf:AllowSharedBufferSource, options?:FileSystemReadWriteOptions|undefined):number;
  truncate(len:number):void;
}
interface CanCreateSyncAccessHandle extends FileSystemFileHandle {
  createSyncAccessHandle():Promise<SyncAccessHandle>;
}

async function writeFile(path:string, data:Uint8Array) : Promise<void> {
  const dir_end = path.lastIndexOf("/");
  const parent = path.substr(0, dir_end);
  const child = path.substr(dir_end+1);
  const parent_dir = await mkdirTree(parent);
  const file = await parent_dir.getFileHandle(child,{create:true});
  const stream = await (file as CanCreateSyncAccessHandle).createSyncAccessHandle();
  stream.write(data);
  stream.close();
}

async function mkdirTree(path:string):Promise<FileSystemDirectoryHandle> {
  const root = await navigator.storage.getDirectory();
  const parts = path.split("/");
  let here = root;
  for (const part of parts) {
    if (part == "") { continue; }
    here = await here.getDirectoryHandle(part, {create:true});
  }
  return here;
}

async function setupZipFS(zipBuf:Uint8Array) : Promise<void> {
  const zipReader = new zip.ZipReader(new zip.Uint8ArrayReader(zipBuf), {useWebWorkers:false});
  const entries = await zipReader.getEntries();
  for(const file of entries) {
    if (file.getData && !file.directory) {
      const writer = new zip.Uint8ArrayWriter();
      const data = await file.getData(writer);
      await writeFile(file.filename, data);
    } else if (file.directory) {
      await mkdirTree(file.filename);
    }
  }
  await zipReader.close();
}

interface SetupMessage {
  command:string,
  time:string,
  gisst_root:string
}

onmessage = async (msg:MessageEvent<SetupMessage>) => {
  if(msg.data.command == "load_bundle") {
    const gisst_root = msg.data.gisst_root;
    let old_timestamp = msg.data.time;
    try {
      const root = await navigator.storage.getDirectory();
      await root.getDirectoryHandle("overlays");
    } catch (_) {
      old_timestamp = "";
    }
    console.log("old timestamp:",old_timestamp);
    const resp = await fetch(gisst_root+"/assets/frontend/assets_minimal.zip", {
      headers: {
        "If-Modified-Since": old_timestamp
      }
    });
    // last-modified will be NULL if we get a 304
    const last_modified = resp.headers.get("last-modified") ?? old_timestamp;
    console.log("resp.status",resp.status,"l-m",last_modified);
    if (resp.status == 200) {
      await setupZipFS(new Uint8Array(await resp.arrayBuffer()));
    } else if (resp.status < 400) {
      await resp.text();
    } else {
      throw resp;
    }
    postMessage({command:"loaded_bundle", time:last_modified});
    close();
  }
}
