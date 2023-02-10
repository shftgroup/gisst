type FileContents = null|Index;
export interface Index extends Record<string, FileContents> {}

export async function registerFetchFS(index:string | Index, root:string, mount:string) {
    let index_file_tree;
    if (typeof index === "string") {
        const index_result = await fetch(index);
        index_file_tree = await index_result.json();
    } else {
        index_file_tree = index;
    }
    console.log("Download files", index_file_tree);
    let promises:Promise<void>[] = [];
    mkdirp(mount);
    fetchDirectory(index_file_tree, root, mount, promises);
    await Promise.all(promises);
}

function fetchDirectory(index_file_tree:Index, root:string, mount:string, promises:Promise<void>[]) {
    for (let file of Object.keys(index_file_tree)) {
        const contents = index_file_tree[file];
        if (contents === null) {
            promises.push(fetchFile((root+"/"+file), mount+"/"+file));
        } else {
            mkdir(mount+"/"+file);
            fetchDirectory(contents, root+"/"+file, mount+"/"+file, promises);
        }
    }
}

function fetchFile(from:string, to:string):Promise<void> {
    return fetch(from).then(r => r.arrayBuffer()).then(b => FS.writeFile(to, new Uint8Array(b)));
}

function mkdir(path:string) {
    try {
        FS.mkdir(path);
    } catch {
        //ignore
    }
}

function mkdirp(path:string) {
    let sofar = [];
    for (let chunk of path.split("/")) {
        sofar.push(chunk);
        mkdir(sofar.join("/"));
    }
}
