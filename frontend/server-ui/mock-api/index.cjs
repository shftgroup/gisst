"use strict";
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// mock-api/index.ts
var index_exports = {};
__export(index_exports, {
  default: () => index_default
});
module.exports = __toCommonJS(index_exports);
var coresHandler = {
  path: "/cores",
  handler: (req, res) => {
    res.json({ cores: [
      { core_name: "fceumm", core_version: "some_hash", core_platform: "Nintendo Entertainment System" },
      { core_name: "gambatte", core_version: "some_hash", core_platform: "Nintendo Game Boy" },
      { core_name: "sameboy", core_version: "some_hash", core_platform: "Nintendo Game Boy" },
      { core_name: "mupen64plus_next", core_version: "some_hash", core_platform: "Nintendo N64" },
      { core_name: "snes9x", core_version: "some_hash", core_platform: "Super Nintendo Entertainment System" },
      { core_name: "pcsx_rearmed", core_version: "some_hash", core_platform: "Sony Playstation" },
      { core_name: "NONE", core_version: "some_hash", core_platform: "Sega Genesis" },
      { core_name: "NONE", core_version: "some_hash", core_platform: "Sega Master System" },
      { core_name: "NONE", core_version: "some_hash", core_platform: "Sega Game Gear" },
      { core_name: "NONE", core_version: "some_hash", core_platform: "Sony Playstation 2" },
      { core_name: "NONE", core_version: "some_hash", core_platform: "Nintendo GameCube" },
      { core_name: "NONE", core_version: "some_hash", core_platform: "Atari 2600" },
      { core_name: "NONE", core_version: "some_hash", core_platform: "Intellivision" }
    ] });
  }
};
var instanceDataHandler = {
  path: "/instances/",
  handler: (req, res) => {
    res.json({
      work: {
        work_platform: "Nintendo Entertainment System",
        work_name: "work b",
        work_version: "0.1",
        work_id: "1"
      },
      info: {
        instance_id: "1",
        work_id: "1",
        environment_id: "1",
        instance_config: {},
        created_on: /* @__PURE__ */ new Date(864e13),
        derived_from_instance: null,
        derived_from_state: null
      },
      environment: {
        enviroment_id: "1",
        environment_name: "work a environment",
        environment_framework: "retroarch",
        environment_core_name: "fceumm",
        environment_core_version: "some_hash",
        environment_derived_from: null,
        environment_config: {},
        created_on: /* @__PURE__ */ new Date(864e13)
      },
      objects: [
        {
          object_id: "o0",
          object_role: "config",
          object_role_index: 0,
          file_hash: "some_hash",
          file_filename: "retroarch.cfg",
          file_source_path: "",
          file_dest_path: ""
        },
        {
          object_id: "o1",
          object_role: "config",
          object_role_index: 1,
          file_hash: "some_hash",
          file_filename: "retroarch-custom.cfg",
          file_source_path: "",
          file_dest_path: ""
        },
        {
          object_id: "o2",
          object_role: "dependency",
          object_role_index: 0,
          file_hash: "some_hash",
          file_filename: "bios.bin",
          file_source_path: "",
          file_dest_path: ""
        },
        {
          object_id: "o3",
          object_role: "content",
          object_role_index: 0,
          file_hash: "some_hash",
          file_filename: "game.nes",
          file_source_path: "",
          file_dest_path: ""
        },
        {
          object_id: "o4",
          object_role: "content",
          object_role_index: 1,
          file_hash: "some_hash",
          file_filename: "game_extra.bin",
          file_source_path: "",
          file_dest_path: ""
        }
      ]
    });
  }
};
var index_default = [coresHandler, instanceDataHandler];
