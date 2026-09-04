import * as ui from "./main";
import { FrontendConfig, Metadata } from "./models";

const IMG_DATA: string =
  "iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAYAAADDPmHLAAABbWlDQ1BpY2MAACiRdZG7S8NQFMZ/poriA0EdRBwyVBFUEAVx1Dp0KSJVwdeSpmkr9BGSFCmugouD4CC6+Br8D3QVXBUEQRFE3Nx9LVLiuUaoiL3h5vz47v0OJ19Ai2XNnFs7Drm858SjEX1+YVGvf0ajkzb6CRuma09MT8eouj5uqVH1ZlD1qn7v39WUtFwTahqER03b8YRlGmKrnq14U7jDzBhJ4QPhAUcGFL5UeiLgJ8XpgN8UO7PxSdBUTz39ixO/2Mw4OeE+4XAuWzR/5lFf0mzl52akdsnuxiVOlAg6CYqskMVjUGpeMvvfN/Ttm6IgHlPeNiUccaTJiHdA1KJ0taSmRLfkyVJSuf/N002NDAfdmyNQ9+j7rz1Qvw3lLd//PPT98hGEHuA8X/EXJKexd9G3Klp4H1rX4fSioiV24GwDOu9twzG+pZBsLZWClxNoWYD2a2hcCrL6Oef4DmbX5Bddwe4e9Mr91uUvPjJoJucWBkIAAAAJcEhZcwAACxIAAAsSAdLdfvwAAAERSURBVHja7dZBCsAgDADBGPz/k2P7gp6aQ3AGehVJV3RFxAmulUYgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACYLjdsWhVmeyfpzRzVgDdm75J92Hyl7wBEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAB82F0LV5XpDrDe7xiDKwABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAAzPMAGvkL/eKCZR0AAAAASUVORK5CYII=";

addEventListener("load", () => {
  let statenum: number = 0;
  let savenum: number = 0;
  let replaynum: number = 0;
  //let cpnum:number = 0;
  const ui_state: ui.UI = new ui.UI(
    <HTMLDivElement>document.getElementById("ui")!,
    false,
  );

  ui_state.emulator_div.style.height = `360px`;
  ui_state.emulator_div.style.backgroundColor = `black`;
  ui_state.emulator_div.style.margin = `10px auto`;

  ui_state.setControl({
    toggle_mute: () => console.log("MUTE/UNMUTE"),
    set_zoom: (lev: ui.ZoomLevel) => {
      let w: number = 0;
      let h: number = 0;
      switch (lev) {
        case ui.ZoomLevel.X05:
          w = 240;
          h = 180;
          break;
        case ui.ZoomLevel.X1:
          w = 480;
          h = 360;
          break;
        case ui.ZoomLevel.X2:
          w = 960;
          h = 720;
          break;
        case ui.ZoomLevel.Fit:
          {
            const bounds = document
              .getElementById("ui")
              ?.getBoundingClientRect();
            const computed_style: CSSStyleDeclaration = window.getComputedStyle(
              ui_state.emulator_div,
            );
            const w_inner: number = bounds?.width || window.innerWidth;
            // Compute inner height minus header, emulator toolbar and emulator window top/bottom margin
            const h_inner: number =
              window.innerHeight -
                (document.getElementById("header")?.getBoundingClientRect()
                  ?.height || 0) -
                (document
                  .getElementById("emulator_control_bar_col")
                  ?.getBoundingClientRect()?.height || 0) -
                parseInt(computed_style.marginTop) -
                parseInt(computed_style.marginBottom) || 0;
            console.log("Width: " + w_inner + " Height: " + h_inner);
            if (h_inner > w_inner) {
              const round_width: number = Math.floor(w_inner / 10) * 10;
              h = Math.round(round_width * 0.75);
              w = round_width;
            } else {
              const round_height: number = Math.floor(h_inner / 10) * 10;
              w = Math.floor(round_height * 1.3333);
              h = round_height;
            }
          }
          break;
        default:
          break;
      }
      console.log("Final Width: " + w + " Final Height: " + h);
      ui_state.emulator_div.style.width = `${w}px`;
      ui_state.emulator_div.style.height = `${h}px`;
      console.log("ZOOM", lev);
    },
    enter_fullscreen: () => console.log("FULLSCREEN"),
    activate_save: (save: string) => console.log("ACTIVATE", save),
    create_save: () => console.log("MAKE SAVE"),
    load_state: (sn: string) => console.log("LOAD", sn),
    save_state: () => {
      ui_state.newState("state" + statenum.toString(), IMG_DATA);
      statenum += 1;
    },
    start_replay: () => {
      ui_state.newReplay("yet another replay.replay" + replaynum.toString());
      replaynum += 1;
    },
    stop_and_save_replay: () => {},
    play_replay: (sn: string) => console.log("PLAY", sn),
    download_file: (category: "save" | "state" | "replay", file_name: string) =>
      console.log("Save file", category, file_name),
    upload_file: (
      category: "save" | "state" | "replay",
      file_name: string,
      metadata: Metadata,
    ) => {
      console.log("Upload file", category, file_name, metadata);
      return new Promise((resolve, reject) => {
        if (metadata) {
          resolve(metadata);
        } else {
          reject("metadata is null");
        }
      });
    },
    checkpoints_of: (_replay: string) => {
      return [];
    },
    evt_to_html: (evt: unknown) => {
      const elt = document.createElement("span");
      elt.innerText = evt as string;
      return elt;
    },
  });
  ui_state.setConfig(
    JSON.parse(
      document.getElementById("config")!.textContent!,
    ) as FrontendConfig,
  );
  ui_state.control.set_zoom(ui.ZoomLevel.X1);

  ui_state.evtlog_append([{ t: 0, evt: "a" }]);
  ui_state.evtlog_append([{ t: 3, evt: "b" }]);
  ui_state.evtlog_set_playhead(3);
  setTimeout(() => {
    ui_state.evtlog_clear();
    ui_state.evtlog_append([
      { t: 0, evt: "c" },
      { t: 2, evt: "d" },
      { t: 4, evt: "e" },
      { t: 6, evt: "f" },
      { t: 8, evt: "g" },
    ]);
    setInterval(() => {
      ui_state.evtlog_set_playhead(ui_state.evtlog_playhead + 1);
    }, 1000);
  }, 3000);

  ui_state.newSave("initial save");
  savenum += 1;
  setInterval(() => {
    savenum++;
    ui_state.newSave(`Save ${savenum}`);
  }, 5000);
});
