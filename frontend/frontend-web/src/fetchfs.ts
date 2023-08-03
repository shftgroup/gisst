import {get, set, delMany} from 'idb-keyval';

type FileContents = null|Index;

export interface Index extends Record<string, FileContents> {}

function min(n:number,m:number) : number {
  if (n <= m) { return n; }
  return m;
}

function get_file_paths_cache(index:Index, files:string[], root:string) {
  for(let file of Object.keys(index)) {
    const contents = index[file];
    if (contents === null) {
      let key = "FSCACHE_"+root+"/"+file;
      files.push(key);
    } else {
      get_file_paths_cache(contents, files, root+"/"+file);
    }
  }
}

function listsEqual(as:any[], bs:any[]):boolean {
  if(as.length != bs.length) { return false; }
  for(let i = 0; i < as.length; i++) {
    if(as[i] != bs[i]) { return false; }
  }
  return true;
}

export async function registerFetchFS(index:string | Index, root:string, mount:string, cache:boolean) {
  let index_file_tree;
  if (typeof index === "string") {
    const index_result = await fetch(index);
    const mod_date = index_result.headers.get("Last-Modified");
    index_file_tree = await index_result.json();
    let last_index_info = await get("FSCACHE_INDEX_"+index);
    if(last_index_info) {
      const last_index = last_index_info[1];
      let old_files:string[] = [];
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
  let files:[string,string][] = [];
  mkdirp(mount);
  fetchDirectory(index_file_tree, root, mount, files);
  const batch_size = 100;
  for(let i = 0; i < files.length; i+=batch_size) {
    let file_batch = files.slice(i,min(i+batch_size,files.length));
    await Promise.all(file_batch.map(([from,to]) => fetchFile(from,to,cache)));
  }
}

function fetchDirectory(index_file_tree:Index, root:string, mount:string, files:[string,string][]) {
  for (let file of Object.keys(index_file_tree)) {
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
    let key = "FSCACHE_"+from;
    let cached = await get(key);
    if (cached) {
      if(to.endsWith("bfight.nes") || to.endsWith("retroarch.cfg")) {
        console.log("make cached req",key,cached.byteLength,from,to);
      }
      return FS.writeFile(to, new Uint8Array(cached));
    } else {
      let resp = await fetch(from);
      let buf = await resp.arrayBuffer();
      set(key,buf);
      if(to.endsWith("bfight.nes") || to.endsWith("retroarch.cfg")) {
        console.log("make uncached req",key,from,to,buf.byteLength);
      }
      return FS.writeFile(to, new Uint8Array(buf));
    }
  } else {
    let resp = await fetch(from);
    let buf = await resp.arrayBuffer();
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
  let sofar = [];
  for (let chunk of path.split("/")) {
    sofar.push(chunk);
    mkdir(sofar.join("/"));
  }
}
