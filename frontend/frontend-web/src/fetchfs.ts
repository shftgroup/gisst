import {get, set, delMany} from 'idb-keyval';
import * as zip from "@zip.js/zip.js";

type FileContents = null|Index;

export interface Index extends Record<string, FileContents> {}

function min(n:number,m:number) : number {
  if (n <= m) { return n; }
  return m;
}

function get_file_paths_cache(index:Index, files:string[], root:string) {
  for(const file of Object.keys(index)) {
    const contents = index[file];
    if (contents === null) {
      const key = "FSCACHE_"+root+"/"+file;
      files.push(key);
    } else {
      get_file_paths_cache(contents, files, root+"/"+file);
    }
  }
}

function listsEqual(as:unknown[], bs:unknown[]):boolean {
  if(as.length != bs.length) { return false; }
  for(let i = 0; i < as.length; i++) {
    if(as[i] != bs[i]) { return false; }
  }
  return true;
}

export async function fetchZip(zipfile:string, mount:string) {
  const zipReader = new zip.ZipReader(new zip.HttpReader(zipfile), {useWebWorkers:false});
  const entries = await zipReader.getEntries();
  mkdirp(mount);
  for(const file of entries) {
    if (file.getData && !file.directory) {
      const writer = new zip.Uint8ArrayWriter();
      const data:Uint8Array = await file.getData(writer);
      await FS.writeFile(mount+file.filename, data);
    } else {
      mkdirp(mount+file.filename);
    }
  }
  await zipReader.close();
}

export async function registerFetchFS(index:string | Index, root:string, mount:string, cache:boolean) {
  let index_file_tree;
  if (typeof index === "string") {
    const index_result = await fetch(index);
    const mod_date = index_result.headers.get("Last-Modified");
    index_file_tree = await index_result.json();
    const last_index_info = await get("FSCACHE_INDEX_"+index);
    if(last_index_info) {
      const last_index = last_index_info[1];
      const old_files:string[] = [];
      get_file_paths_cache(last_index, old_files, root);
      const new_files:string[] = [];
      get_file_paths_cache(index_file_tree, new_files, root);
      if(last_index_info[0] != mod_date || !listsEqual(old_files, new_files)) {
        console.log("Asset cache out of date, mark all assets for redownload",old_files.length,old_files[0]);
        await delMany(old_files);
      }
    }
    set("FSCACHE_INDEX_"+index, [mod_date, index_file_tree]);
  } else {
    index_file_tree = index;
  }
  const files:[string,string][] = [];
  mkdirp(mount);
  fetchDirectory(index_file_tree, root, mount, files);
  const batch_size = 100;
  for(let i = 0; i < files.length; i+=batch_size) {
    const file_batch = files.slice(i,min(i+batch_size,files.length));
    await Promise.all(file_batch.map(([from,to]) => fetchFile(from,to,cache)));
  }
}

function fetchDirectory(index_file_tree:Index, root:string, mount:string, files:[string,string][]) {
  for (const file of Object.keys(index_file_tree)) {
    const contents = index_file_tree[file];
    if (contents === null) {
      files.push([root+"/"+file, mount+"/"+file]);
    } else {
      mkdir(mount+"/"+file);
      fetchDirectory(contents, root+"/"+file, mount+"/"+file, files);
    }
  }
}

export async function fetchFile(from:string, to:string, cache:boolean):Promise<void> {
  if (cache) {
    const key = "FSCACHE_"+from;
    const cached = await get(key);
    if (cached) {
      return FS.writeFile(to, new Uint8Array(cached));
    } else {
      const resp = await fetch(from);
      if(!resp.ok) {
        throw "Couldn't obtain file";
      }
      const buf = await resp.arrayBuffer();
      set(key,buf);
      return FS.writeFile(to, new Uint8Array(buf));
    }
  } else {
    const resp = await fetch(from);
    if(!resp.ok) {
      throw "Couldn't obtain file";
    }
    const buf = await resp.arrayBuffer();
    return FS.writeFile(to, new Uint8Array(buf));
  }
}

export function mkdir(path:string) {
  try {
    FS.mkdir(path);
  } catch {
    //ignore
  }
}

export function mkdirp(path:string) {
  const sofar = [];
  for (const chunk of path.split("/")) {
    sofar.push(chunk);
    mkdir(sofar.join("/"));
  }
}
