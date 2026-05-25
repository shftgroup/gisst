import * as fs from 'fs';

if (fs.existsSync("mock-data/assets/retroarch_web_base.cfg")) {
  process.exit(0);
}

fs.mkdirSync("mock-data/storage/e/2/7/9",{recursive:true});
fs.copyFileSync("../../build/libv86.js", "mock-data/storage/e/2/7/9/c750f9b93bca846d2b583db91380a25c-libv86.js");
fs.mkdirSync("mock-data/storage/1/1/a/7",{recursive:true});
fs.copyFileSync("../../build/bochs-bios.bin", "mock-data/storage/1/1/a/7/fd359ac2ee5e1023d414924e4ea59db2-bochs-bios.bin");
fs.mkdirSync("mock-data/storage/1/4/5/4",{recursive:true});
fs.copyFileSync("../../build/bochs-vgabios.bin", "mock-data/storage/1/4/5/4/ae8105414e5c898059dae94db5b95b60-bochs-vgabios.bin");
fs.mkdirSync("mock-data/storage/5/7/a/d",{recursive:true});
fs.copyFileSync("../../build/vgabios.bin", "mock-data/storage/5/7/a/d/e8acaa929079296b3ca7c691ddc30914-vgabios.bin");
fs.mkdirSync("mock-data/storage/a/9/f/e",{recursive:true});
fs.copyFileSync("../../backend/examples/data/nes/Alter Ego.nes", "mock-data/storage/a/9/f/e/1f21f648af663fbd3c864b7283d994c7-Alter Ego.nes");
fs.mkdirSync("mock-data/storage/9/a/5/c",{recursive:true});
fs.copyFileSync("../../build/cores/fceumm_libretro.js", "mock-data/storage/9/a/5/c/2703f155daacf3377f4cc0f5a8df1965-fceumm_libretro.js");
fs.mkdirSync("mock-data/storage/7/1/c/0",{recursive:true});
fs.copyFileSync("../../build/cores/fceumm_libretro.wasm", "mock-data/storage/7/1/c/0/c09ccd0799c1b369a0812884a4c064cb-fceumm_libretro.wasm");
fs.mkdirSync("mock-data/storage/6/d/7/3",{recursive:true});
fs.copyFileSync("../../build/seabios.bin", "mock-data/storage/6/d/7/3/ed52c23d996a549f7cecee6a540bd728-seabios.bin");
fs.mkdirSync("mock-data/storage/8/5/7/4",{recursive:true});
fs.copyFileSync("../../build/v86.wasm", "mock-data/storage/8/5/7/4/d68aff64381e1782fbb128b79402ec1e-v86.wasm");
fs.mkdirSync("mock-data/storage/b/7/3/6",{recursive:true});
fs.copyFileSync("../../backend/examples/data/v86/freedos722.img", "mock-data/storage/b/7/3/6/07cd9656778aa01e7f99f37e31b76e24-freedos722.img");
fs.mkdirSync("mock-data/assets/frontend",{recursive:true});
fs.copyFileSync("../../build/assets_minimal.zip", "mock-data/assets/frontend/assets_minimal.zip");
fs.copyFileSync("../frontend-web/public/assets/retroarch_web_base.cfg", "mock-data/assets/retroarch_web_base.cfg");
