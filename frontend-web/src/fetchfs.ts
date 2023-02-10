type FileContents = null|Index;
export interface Index extends Record<string, FileContents> {}

function min(n:number,m:number) : number {
    if (n <= m) { return n; }
    return m;
}

export async function registerFetchFS(index:string | Index, root:string, mount:string) {
    let index_file_tree;
    if (typeof index === "string") {
        const index_result = await fetch(index);
        index_file_tree = await index_result.json();
    } else {
        index_file_tree = index;
    }
    // TODO: keep our own cache so that we don't need to make 5000 requests.
    // OR, download and unzip a bundle of assets.
    // OR, just remove overlays and unnecessary GUI icons and stuff from the bundles
    let files:[string,string][] = [];
    mkdirp(mount);
    fetchDirectory(index_file_tree, root, mount, files);
    const batch_size = 100;
    for(let i = 0; i < files.length; i+=batch_size) {
        let file_batch = files.slice(i,min(i+batch_size,files.length));
        await Promise.all(file_batch.map(([from,to]) => fetchFile(from,to)));
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

function fetchFile(from:string, to:string):Promise<void> {
    return fetch(from).then(r => r.arrayBuffer()).then(b => FS.writeFile(to, new Uint8Array(b)));
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
