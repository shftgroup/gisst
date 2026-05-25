import {
  MockHandler,
  MockApiHandlerRequest,
  MockApiHandlerResponse,
  setJSON
} from "vite-mock-api";

let upload_length = 0;

const resourcesHandler:MockHandler = {
  path:"/resources/",
  handler: (req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    upload_length = parseInt(req.headers["upload-length"] || "0");
    res.writeHead(201, {
      "Location":"/resources/96f7f017-0de4-4f03-9696-96c89d16c9d1",
      "Cache-Control":"no-store",
      "Upload-Length":upload_length,
      "Tus-Resumable": "1.0.0"
    });
    res.end();
  }
}
const resourcesPatchHandler:MockHandler = {
  path:"/resources/96f7f017-0de4-4f03-9696-96c89d16c9d1",
  handler: (req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    res.writeHead(204, {
      "Cache-Control":"no-store",
      "Upload-Offset":upload_length,
      "Upload-Length":upload_length,
      "Tus-Resumable": "1.0.0"
    });
    res.end();
  }
}
// modify for state, save, replay
const replayCreateHandler:MockHandler = {
  path: "/replays/create",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    setJSON(res,{replay_id:"96f7f017-0de4-4f03-9696-96c89d16c9df"});
    res.end();
  }
};
const saveCreateHandler:MockHandler = {
  path: "/saves/create",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    setJSON(res,{save_id:"96f7f017-0de4-4f03-9696-96c89d16c9de"});
    res.end();
  }
};
const screenshotCreateHandler:MockHandler = {
  path: "/screenshots/create",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    setJSON(res,{screenshot_id:"96f7f017-0de4-4f03-9696-96c89d16c9da"});
    res.end();
  }
};
const stateCreateHandler:MockHandler = {
  path: "/states/create",
  handler: (_req:MockApiHandlerRequest, res:MockApiHandlerResponse) => {
    setJSON(res,{state_id:"96f7f017-0de4-4f03-9696-96c89d16c9dc"});
    res.end();
  }
};
export default [resourcesPatchHandler, resourcesHandler, replayCreateHandler, screenshotCreateHandler, saveCreateHandler, stateCreateHandler];
