import * as zip from "@zip.js/zip.js";
import {LibretroModule} from './libretro_adapter';

type FileContents = null|Index;

export interface Index extends Record<string, FileContents> {}

function min(n:number,m:number) : number {
  if (n <= m) { return n; }
  return m;
}

export async function fetchZip(module:LibretroModule, zipfile:string, mount:string) {
  const zipReader = new zip.ZipReader(new zip.HttpReader(zipfile), {useWebWorkers:false});
  const entries = await zipReader.getEntries();
  mkdirp(module,mount);
  for(const file of entries) {
    if (file.getData && !file.directory) {
      const writer = new zip.Uint8ArrayWriter();
      const data:Uint8Array = await file.getData(writer);
      await module.FS.writeFile(mount+file.filename, data);
    } else {
      mkdirp(module,mount+file.filename);
    }
  }
  await zipReader.close();
}

export async function registerFetchFS(module:LibretroModule, index:string | Index, root:string, mount:string) {
  let index_file_tree;
  if (typeof index === "string") {
    const index_result = await fetch(index);
    index_file_tree = await index_result.json();
  } else {
    index_file_tree = index;
  }
  const files:[string,string][] = [];
  mkdirp(module,mount);
  fetchDirectory(module,index_file_tree, root, mount, files);
  const batch_size = 100;
  for(let i = 0; i < files.length; i+=batch_size) {
    const file_batch = files.slice(i,min(i+batch_size,files.length));
    await Promise.all(file_batch.map(([from,to]) => fetchFile(module,from,to)));
  }
}

function fetchDirectory(module:LibretroModule,index_file_tree:Index, root:string, mount:string, files:[string,string][]) {
  for (const file of Object.keys(index_file_tree)) {
    const contents = index_file_tree[file];
    if (contents === null) {
      files.push([root+"/"+file, mount+"/"+file]);
    } else {
      mkdir(module,mount+"/"+file);
      fetchDirectory(module,contents, root+"/"+file, mount+"/"+file, files);
    }
  }
}

export async function fetchFile(module:LibretroModule,from:string, to:string):Promise<void> {
  const resp = await fetch(from);
  if(!resp.ok) {
    throw "Couldn't obtain file";
  }
  const buf = await resp.arrayBuffer();
  return module.FS.writeFile(to, new Uint8Array(buf));
}

export function mkdir(module:LibretroModule,path:string) {
  try {
    module.FS.mkdir(path);
  } catch {
    //ignore
  }
}

export function mkdirp(module:LibretroModule, path:string) {
  const sofar = [];
  for (const chunk of path.split("/")) {
    sofar.push(chunk);
    mkdir(module,sofar.join("/"));
  }
}
