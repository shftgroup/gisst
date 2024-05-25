#!/usr/bin/env node
"use strict";

var fs = require("fs");
var path = require("path");

function readfile(path)
{
    return new Uint8Array(fs.readFileSync(path)).buffer;
}

var V86Starter = require("../web-dist/v86/libv86.js").V86Starter;

// console.log("Now booting, please stand by ...");

// let state=readfile(__dirname+"/../storage/0/3/1/e/087bbdc6dbb84234b43ce0b562d8486c-state2.v86state");
let state=readfile("/home/jcoa2018/Downloads/state1.v86state");
var emulator = new V86Starter({
  bios: { url:path.join(__dirname, "../web-dist/v86/bios/seabios.bin") },
  vga_bios: { url:path.join(__dirname, "../web-dist/v86/bios/vgabios.bin") },
  fda: { url:path.join(__dirname, "../storage/7/6/d/5/07cd9656778aa01e7f99f37e31b76e24-freedos722.img"), async:true },
  autostart: false,
});
emulator.add_listener("emulator-loaded", async function(){
  // console.log("load state")
  emulator.stop();
  await emulator.restore_state(state);
  // console.log(emulator.disk_images);
  emulator.run();
  let out = fs.mkdtempSync("out");
  const disks = ["fda", "hda"];
  for (let disk of disks) {
    let buf = emulator.disk_images[disk];
    if (buf == null) { continue; }
    let in_path = buf.filename;
    // TODO make randomized name?
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
    console.log(out_path);
  }
  process.exit(0);
});
emulator.run();

