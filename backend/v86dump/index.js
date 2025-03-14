#!/usr/bin/env node
"use strict";

var fs = require("fs");
var path = require("path");

var args = process.argv.slice(2);
if (args.length != 2) {
  console.error("Usage: v86-json state-path");
  process.exit(1);
}
var emu_info = JSON.parse(args[0]);
var state_path = args[1];

const disks = ["fda", "hda"];
const bios = ["bios", "vgabios"];

function readfile(path)
{
    return new Uint8Array(fs.readFileSync(path)).buffer;
}

var V86 = require("../web-dist/libv86.js").V86;

// console.log("Now booting, please stand by ...");

// let state=readfile(__dirname+"/../storage/0/3/1/e/087bbdc6dbb84234b43ce0b562d8486c-state2.v86state");
let state=readfile(state_path);
var emulator = new V86(emu_info);
emulator.add_listener("emulator-loaded", async function(){
  // console.log("load state")
  emulator.stop();
  await emulator.restore_state(state);
  // console.log(emulator.disk_images);
  emulator.run();
  let out = fs.mkdtempSync("out");
  for (let disk of disks) {
    let buf = emulator.disk_images[disk];
    if (buf == null) {
      continue;
    }
    let in_path = buf.filename;
    let filename = path.basename(in_path);
    filename = filename.substring(filename.indexOf("-")+1);
    let out_path = path.join(out,filename);
    fs.copyFileSync(in_path, out_path);
    let outfile = fs.openSync(out_path, 'r+');
    for(let [index, block] of buf.block_cache)
    {
      if(buf.block_cache_is_write.has(index))
      {
        fs.writeSync(outfile, block, 0, block.length, index*256)
      }
    }
    fs.fsyncSync(outfile);
    console.log(disk+":"+out_path);
  }
  process.exit(0);
});
emulator.run();

