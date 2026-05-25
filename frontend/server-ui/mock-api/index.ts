import {
  MockHandler,
  MockApiHandlerRequest,
  MockApiHandlerResponse,
  setJSON
} from "vite-mock-api";

const coresHandler:MockHandler = {
  path: "/cores",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    setJSON(res,{cores:[
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
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    setJSON(res,{
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
const resourcesHandler:MockHandler = {
  path:"/resources/",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    res.statusCode = 201;
    res.setHeader('Location','/resources/96f7f017-0de4-4f03-9696-96c89d16c9d1');
    res.end();
  }
}
const resourcesPatchHandler:MockHandler = {
  path:"/resources/96f7f017-0de4-4f03-9696-96c89d16c9d1",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    res.statusCode = 204;
  }
}
const objectCreateHandler:MockHandler = {
  path: "/objects/create",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    setJSON(res,{object_id:"96f7f017-0de4-4f03-9696-96c89d16c9d0"});
  }
};

const environmentCreateHandler:MockHandler = {
  path: "/environments/create",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    setJSON(res,{environment_id:"96f7f017-0de4-4f03-9696-96c89d16c9dd"});
  }
};
const workCreateHandler:MockHandler = {
  path: "/works/create",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    setJSON(res,{work_id:"96f7f017-0de4-4f03-9696-96c89d16c9de"});
  }
};
const instanceCreateHandler:MockHandler = {
  path: "/instances/create",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    setJSON(res,{instance_id:"96f7f017-0de4-4f03-9696-96c89d16c9df"});
  }
};
const lookupWorkHandler:MockHandler = {
  path: "/lookup-work",
  handler: (req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    if (req.params!["filename"] == "existing-work.bin") {
      setJSON(res,{
        work_platform:"Nintendo Entertainment System",
        work_name:"work b",
        work_version:"0.1",
        work_id:"1",
        work_derived_from: null,
        instance_id:"1"
      });
    } else if (req.params!["filename"] == "matched-work.bin") {
      setJSON(res,{
        work_name: req.params!["filename"],
        work_platform: req.params!["platform"],
        work_version: "1.0",
        work_derived_from: null
      });
    } else {
      res.statusCode = 404;
      res.end();
    }
  }
};

export default [coresHandler,instanceDataHandler,objectCreateHandler,environmentCreateHandler,workCreateHandler,instanceCreateHandler,lookupWorkHandler,resourcesPatchHandler,resourcesHandler];
