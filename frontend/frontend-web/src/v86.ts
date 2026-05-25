import {GISSTDBConnector, GISSTModels, ReplayMode as UIReplayMode, ZoomLevel, UI} from 'gisst-player';
import {saveAs} from './util';
import {EmbedV86,StateInfo,ReplayEvent} from 'embedv86';
import {StateStart, ReplayStart, EmuControls} from 'frontend-embed/types';
let ui_state:UI;
let db:GISSTDBConnector;
let zoom_fit = false;
let current_zoom = 1.0;
let v86:EmbedV86;

export async function init(ui:UI, embed:EmuControls) {
  ui_state = ui;
  db = new GISSTDBConnector(embed.gisst_root);
  v86 = embed.inner;
  let entry_state:boolean = false;
  let entry_screenshot:string|null = null;
  if (embed.start.type == "state") {
    const data = (embed.start as StateStart).data;
    if(!data.screenshot_id) {
      console.error("No screenshot for entry state");
      entry_screenshot = "";
    } else {
      const screenshot_data = (await db.getRecordById("screenshot", data.screenshot_id)) as GISSTModels.Screenshot;
      entry_screenshot = screenshot_data.screenshot_data;
    }
    entry_state = true;
  }
  const movie = embed.start.type == "replay";
  const state_to_replay:Array<number|null> = [];
  const EvtNames:string[] = ["keyboard-code", "mouse-click", "mouse-delta", "mouse-absolute", "mouse-wheel"];
  let evtlog_idx = 0;
  function fill_evtlog(fromidx:number, toidx:number) {
    if (v86.active_replay === null) { return; }
    const replay = v86.replays[v86.active_replay];
    ui_state.evtlog_append(replay.events.slice(fromidx,toidx).map((evt) => {return {t:evt.when,evt}}))
  }
  function zoom_to(n:number) {
    if (n == current_zoom) { return; }
    current_zoom = n;
    v86.emulator.screen_set_scale(n,n);
  }
  function zoom_to_fit() {
    const container = <HTMLDivElement>document.querySelector(".gisst-internal-content-body")!;
    const screen = <HTMLDivElement>ui_state.emulator_div.querySelector("canvas")!.parentElement!.parentElement!;
    const rect = screen.getBoundingClientRect();
    const n = Math.min(container.clientWidth/rect.width,container.clientHeight/rect.height);
    zoom_to(n);
  }
  ui_state.setControl(
    {
      "evt_to_html": (evt_:unknown) => {
        const evt = evt_ as ReplayEvent;
        const elt = document.createElement("span");
        elt.innerText = `${EvtNames[evt.code]} ${evt.value}`;
        return elt;
      },
      "toggle_mute": () => {
        embed.toggle_mute();
      },
      "set_zoom": (level:ZoomLevel) => {
        zoom_fit = level == ZoomLevel.Fit;
        switch(level) {
          case ZoomLevel.X05:
            zoom_to(0.5);
            break;
          case ZoomLevel.X1:
            zoom_to(1);
            break;
          case ZoomLevel.X2:
            zoom_to(2);
            break;
          case ZoomLevel.Fit:
            zoom_to_fit();
            break;
        }
      },
      "enter_fullscreen": () => {
        zoom_fit = false;
        v86.emulator.screen_set_scale(1,1);
        const container = <HTMLDivElement>document.getElementById("canvas_div")!;
        container.requestFullscreen().then(() => {
          // const n = Math.min(container.clientWidth/window.innerWidth,container.clientHeight/window.innerHeight);
          // zoom_to(n);
          v86.emulator.lock_mouse();
        });
      },
      "activate_save": (_savefile) => {},
      "create_save": () => {},
      "load_state":(n:number) => {
        (async () => {
          // get the replay of state n
          const replay = state_to_replay[n];
          // if it's not the same as the active replay we have to do something
          if(replay !== v86.active_replay) {
            await v86.stop_replay();
            if(replay != null) {
              await v86.play_replay_slot(replay);
            }
          }
          await v86.load_state_slot(n);
        })();
      },
      "save_state":() => {
        v86.save_state()
      },
      "start_replay":() => {
        // clear evt log and fill and update playhead
        v86.record_replay().then(() => {
          ui_state.evtlog_clear();
          ui_state.evtlog_set_playhead(0);
          evtlog_idx = 0;
        });
      },
      "stop_and_save_replay":() => {
        v86.stop_replay()
      },
      "play_replay":(n:number) => {
        // clear evt log and fill and update playhead
        v86.play_replay_slot(n).then(() => {
          ui_state.evtlog_clear();
          ui_state.evtlog_set_playhead(0);
          evtlog_idx = 0;
          fill_evtlog(0,v86.replays[v86.active_replay!].events.length);
        });
      },
      "download_file":(category:"state" | "save" | "replay", file_name:string) => {
        v86.download_file(category, file_name).then(([blob,name]) => saveAs(blob,name));
      },
      "checkpoints_of":(replay_slot:number) => {
        const cps = v86.replays[replay_slot].checkpoints;
        // don't provide the first and last checkpoint since those are
        // made implicitly within the replay and never would have been
        // added via replay_checkpoints_changed --- investigate this
        // for later or only upload ones we've witnessed through
        // replay_checkpoints_changed
        const rep = cps.slice(1, cps.length-1);
        return rep.map((cp) => cp.name);
      },
      "upload_file":(category:"state" | "save" | "replay", file_name:string, metadata:GISSTModels.Metadata) => {
        return new Promise((resolve, reject) => {
          v86.download_file(category, file_name).then(([blob, name]) => {
            db.uploadFile(new File([blob], name), metadata.record.file_id,
              (error:Error) => { reject(error.message)},
              (_percentage:number) => {},
              (uuid_string:string) => {
                metadata.record.file_id = uuid_string;
                if(category == "state"){
                  db.uploadRecord({screenshot_data: metadata.screenshot.split(",")[1]}, "screenshot")
                    .then((screenshot:GISSTModels.DBRecord) => {
                      (metadata.record as GISSTModels.State).screenshot_id = (screenshot as GISSTModels.Screenshot).screenshot_id;
                      console.log(metadata);
                      db.uploadRecord(metadata.record, "state")
                        .then((state:GISSTModels.DBRecord) => {
                          (metadata.record as GISSTModels.State).state_id = (state as GISSTModels.State).state_id
                          resolve(metadata);
                        })
                        .catch(() => reject("State upload from v86 failed."))
                    })
                    .catch(() => reject("Screenshot upload from v86 failed."))
                }else {
                  db.uploadRecord(metadata.record, category)
                    .then((record:GISSTModels.DBRecord) => {
                      if (category === "replay") {
                        (metadata.record as GISSTModels.Replay).replay_id = (record as GISSTModels.Replay).replay_id;
                      } else {
                        (metadata.record as GISSTModels.Save).save_id = (record as GISSTModels.Save).save_id;
                      }
                      resolve(metadata)
                    })
                    .catch(() => reject(category + " upload from v86 failed"))
                }
              }
            )
              .catch(() => reject("File upload from v86 failed."))
          })
        })
      }
    }
  );
  const container = embed.container;
  const ro = new ResizeObserver((_entries, _observer) => {
    if (!zoom_fit) { return; }
    zoom_to_fit();
  });
  ro.observe(container.parentElement!);
  v86 = embed.inner as EmbedV86;
  v86.set_register_replay((nom:string) => {
    if(movie && nom == "replay0") {
      const data = (embed.start as ReplayStart).data;
      ui_state.newReplay(nom, "init", data as GISSTModels.ReplayFileLink);
    } else {
      ui_state.newReplay(nom, nom);
    }
  });
  v86.set_states_changed((added:StateInfo[], _removed:StateInfo[]) => {
    for(const si of added) {
      if(entry_state && si.name == "state0") {
        si.thumbnail = entry_screenshot!;
        const data = (embed.start as StateStart).data;
        ui_state.newState(si.name, si.thumbnail, "init", data as GISSTModels.StateFileLink);
        state_to_replay.push(null);
      } else {
        ui_state.newState(si.name,si. thumbnail, "no replay");
      }
    }
  });
  v86.set_replay_checkpoints_changed((added:StateInfo[], removed:StateInfo[]) => {
    for(const si of removed) {
      ui_state.removeState(si.name);
    }
    for(const si of added) {
      const checkpoint_matches = si.name.match(/replay([0-9a-f-]+)-state([0-9]+)/);
      if (checkpoint_matches == null) {
        throw "added checkpoint with bad name format";
      }
      const replay_idx = v86.replays.findIndex((elt) => elt.id == checkpoint_matches[1]);
      ui_state.newState(si.name,si.thumbnail,"replay"+String(replay_idx));
      state_to_replay.push(replay_idx);
    }
  });
  setInterval(() => {
    if(v86.active_replay === null) {
      ui_state.setReplayMode(UIReplayMode.Inactive);
    } else {
      const replay = v86.replays[v86.active_replay];
      const mode = replay.mode as UIReplayMode;
      ui_state.setReplayMode(mode);
      // if recording, append since last index and update playhead to t
      if (mode == UIReplayMode.Record) {
        fill_evtlog(evtlog_idx, replay.events.length);
        evtlog_idx = replay.events.length;
      }
      // console.log("ph",replay.current_time(),ui_state.evtlog_playhead,ui_state.evtlog_playhead_eltidx);
      ui_state.evtlog_set_playhead(replay.current_time());
    }
  }, 250);
}
