import * as ui from './main';
import {FrontendConfig, Metadata} from "./models";

const IMG_DATA:string = "iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAYAAADDPmHLAAABbWlDQ1BpY2MAACiRdZG7S8NQFMZ/poriA0EdRBwyVBFUEAVx1Dp0KSJVwdeSpmkr9BGSFCmugouD4CC6+Br8D3QVXBUEQRFE3Nx9LVLiuUaoiL3h5vz47v0OJ19Ai2XNnFs7Drm858SjEX1+YVGvf0ajkzb6CRuma09MT8eouj5uqVH1ZlD1qn7v39WUtFwTahqER03b8YRlGmKrnq14U7jDzBhJ4QPhAUcGFL5UeiLgJ8XpgN8UO7PxSdBUTz39ixO/2Mw4OeE+4XAuWzR/5lFf0mzl52akdsnuxiVOlAg6CYqskMVjUGpeMvvfN/Ttm6IgHlPeNiUccaTJiHdA1KJ0taSmRLfkyVJSuf/N002NDAfdmyNQ9+j7rz1Qvw3lLd//PPT98hGEHuA8X/EXJKexd9G3Klp4H1rX4fSioiV24GwDOu9twzG+pZBsLZWClxNoWYD2a2hcCrL6Oef4DmbX5Bddwe4e9Mr91uUvPjJoJucWBkIAAAAJcEhZcwAACxIAAAsSAdLdfvwAAAERSURBVHja7dZBCsAgDADBGPz/k2P7gp6aQ3AGehVJV3RFxAmulUYgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACYLjdsWhVmeyfpzRzVgDdm75J92Hyl7wBEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAB82F0LV5XpDrDe7xiDKwABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAAzPMAGvkL/eKCZR0AAAAASUVORK5CYII=";

addEventListener("load", () =>
  {
    let statenum:number = 0;
    let savenum:number = 0;
    let replaynum:number = 0;
    //let cpnum:number = 0;
    const ui_state:ui.UI<string> = new ui.UI(
      <HTMLDivElement>document.getElementById("ui")!,
      {
        toggle_mute: () => console.log("MUTE/UNMUTE"),
        activate_save: (save:string) => console.log("ACTIVATE",save),
        create_save: () => console.log("MAKE SAVE"),
        load_state: (sn:number) => console.log("LOAD",sn),
        save_state: () => {
          ui_state.newState("state"+statenum.toString(), IMG_DATA);
          statenum += 1;
        },
        start_replay: () => {
          ui_state.newReplay("yet another replay.replay"+replaynum.toString());
          replaynum +=1;

        },
        stop_and_save_replay: () => {},
        play_replay: (sn:number) => console.log("PLAY",sn),
        download_file: (category:"save"|"state"|"replay", file_name:string) => console.log("Save file",category,file_name),
        upload_file: (category:"save"|"state"|"replay", file_name:string, metadata:Metadata) => {
          console.log("Upload file", category, file_name, metadata);
          return new Promise((resolve, reject) => {
            if (metadata) { resolve(metadata); }
            else { reject("metadata is null"); }
          });
        },
        checkpoints_of: (_replay:number) => {return []},
        evt_to_html: (evt:string) => {
          const elt = document.createElement("span");
          elt.innerText=evt;
          return elt;
        }
      },
      false,
      JSON.parse(document.getElementById("config")!.textContent!) as FrontendConfig
    );

    ui_state.evtlog_append([{t:0, evt:"a"}]);
    ui_state.evtlog_append([{t:3, evt:"b"}]);
    ui_state.evtlog_set_playhead(3);
    setTimeout(() => {
      ui_state.evtlog_clear();
      ui_state.evtlog_append([
        {t:0, evt:"c"},
        {t:2, evt:"d"},
        {t:4, evt:"e"},
        {t:6, evt:"f"},
        {t:8, evt:"g"},
      ]);
      setInterval(() => {
        ui_state.evtlog_set_playhead(ui_state.evtlog_playhead+1);
      }, 1000);
    }, 3000);

    ui_state.newSave("initial save");
    savenum += 1;
    setInterval(() => {
      savenum++;
      ui_state.newSave(`Save ${savenum}`)
    }, 5000);
  });
