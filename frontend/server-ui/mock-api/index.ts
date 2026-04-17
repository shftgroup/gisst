import {
  MockHandler,
  MockApiHandlerRequest,
  MockApiHandlerResponse
} from "vite-mock-api";

const coresHandler:MockHandler = {
  path: "/cores",
  handler: (req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    res.json({cores:[
      {core_name:"fceumm", core_version:"some_hash", core_platform:"Nintendo Entertainment System"},
      {core_name:"gambatte", core_version:"some_hash", core_platform:"Nintendo Game Boy"},
      {core_name:"sameboy", core_version:"some_hash", core_platform:"Nintendo Game Boy"},
      {core_name:"mupen64plus_next", core_version:"some_hash", core_platform:"Nintendo N64"},
      {core_name:"snes9x", core_version:"some_hash", core_platform:"Super Nintendo Entertainment System"},
      {core_name:"pcsx_rearmed", core_version:"some_hash", core_platform:"Sony Playstation"},
      {core_name:"NONE", core_version:"some_hash", core_platform:"Sega Genesis"},
      {core_name:"NONE", core_version:"some_hash", core_platform:"Sega Master System"},
      {core_name:"NONE", core_version:"some_hash", core_platform:"Sega Game Gear"},
      {core_name:"NONE", core_version:"some_hash", core_platform:"Sony Playstation 2"},
      {core_name:"NONE", core_version:"some_hash", core_platform:"Nintendo GameCube"},
      {core_name:"NONE", core_version:"some_hash", core_platform:"Atari 2600"},
      {core_name:"NONE", core_version:"some_hash", core_platform:"Intellivision"},
    ]});
  }
};
const instanceDataHandler:MockHandler = {
  path: "/instances/",
  handler: (req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    res.json({
      work:{
        work_platform:"Nintendo Entertainment System",
        work_name:"work b",
        work_version:"0.1",
        work_id:"1"
      },
      info:{
        instance_id:"1",
        work_id:"1",
        environment_id: "1",
        instance_config: {},
        created_on: new Date(8.64e15),
        derived_from_instance: null,
        derived_from_state: null
      },
      environment:{
        enviroment_id: "1",
        environment_name: "work a environment",
        environment_platform: "Nintendo Entertainment System",
        environment_framework: "retroarch",
        environment_core_name: "fceumm",
        environment_core_version: "some_hash",
        environment_derived_from: null,
        environment_config: {},
        created_on: new Date(8.64e15)
      },
      objects:[
        {
          object_id:"o0",
          object_role:"config",
          object_role_index:0,
          file_hash:"some_hash",
          file_filename:"retroarch.cfg",
          file_source_path:"",
          file_dest_path:"",
        },
        {
          object_id:"o1",
          object_role:"config",
          object_role_index:1,
          file_hash:"some_hash",
          file_filename:"retroarch-custom.cfg",
          file_source_path:"",
          file_dest_path:"",
        },
        {
          object_id:"o2",
          object_role:"dependency",
          object_role_index:0,
          file_hash:"some_hash",
          file_filename:"bios.bin",
          file_source_path:"",
          file_dest_path:"",
        },
        {
          object_id:"o3",
          object_role:"content",
          object_role_index:0,
          file_hash:"some_hash",
          file_filename:"game.nes",
          file_source_path:"",
          file_dest_path:"",
        },
        {
          object_id:"o4",
          object_role:"content",
          object_role_index:1,
          file_hash:"some_hash",
          file_filename:"game_extra.bin",
          file_source_path:"",
          file_dest_path:"",
        },
      ],
    });
  }
};
export default [coresHandler,instanceDataHandler];
