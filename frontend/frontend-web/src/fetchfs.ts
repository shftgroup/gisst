import * as zip from "@zip.js/zip.js";
import {LibretroModule} from './libretro_adapter';

type FileContents = null|Index;

export interface Index {
  [loc: string]: FileContents;
}

export async function fetchZip(module:LibretroModule, zipfile:string, mount:string) {
  const zipReader = new zip.ZipReader(new zip.HttpReader(zipfile), {useWebWorkers:false});
  const entries = await zipReader.getEntries();
  for(const file of entries) {
    if (file.getData && !file.directory) {
      const writer = new zip.Uint8ArrayWriter();
      const data:Uint8Array = await file.getData(writer);
      module.FS.createPreloadedFile(mount+file.filename, undefined, data, true, true);
    } else if (file.directory) {
      module.FS.createPath(mount, file.filename, true, true);
    }
  }
  await zipReader.close();
}

export function fetchFile(module:LibretroModule,from:string, to:string) {
  module.FS.createPreloadedFile(to, undefined, from, true, true);
}

