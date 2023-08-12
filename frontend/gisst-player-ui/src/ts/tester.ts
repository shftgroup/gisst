import * as ui from './main';
import {UIIDConst} from "./template_consts";

const IMG_DATA:string = "iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAYAAADDPmHLAAABbWlDQ1BpY2MAACiRdZG7S8NQFMZ/poriA0EdRBwyVBFUEAVx1Dp0KSJVwdeSpmkr9BGSFCmugouD4CC6+Br8D3QVXBUEQRFE3Nx9LVLiuUaoiL3h5vz47v0OJ19Ai2XNnFs7Drm858SjEX1+YVGvf0ajkzb6CRuma09MT8eouj5uqVH1ZlD1qn7v39WUtFwTahqER03b8YRlGmKrnq14U7jDzBhJ4QPhAUcGFL5UeiLgJ8XpgN8UO7PxSdBUTz39ixO/2Mw4OeE+4XAuWzR/5lFf0mzl52akdsnuxiVOlAg6CYqskMVjUGpeMvvfN/Ttm6IgHlPeNiUccaTJiHdA1KJ0taSmRLfkyVJSuf/N002NDAfdmyNQ9+j7rz1Qvw3lLd//PPT98hGEHuA8X/EXJKexd9G3Klp4H1rX4fSioiV24GwDOu9twzG+pZBsLZWClxNoWYD2a2hcCrL6Oef4DmbX5Bddwe4e9Mr91uUvPjJoJucWBkIAAAAJcEhZcwAACxIAAAsSAdLdfvwAAAERSURBVHja7dZBCsAgDADBGPz/k2P7gp6aQ3AGehVJV3RFxAmulUYgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACYLjdsWhVmeyfpzRzVgDdm75J92Hyl7wBEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAB82F0LV5XpDrDe7xiDKwABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAAzPMAGvkL/eKCZR0AAAAASUVORK5CYII=";

const EMULATOR_WINDOW_STYLE:string = "height: 600px;";

addEventListener("load", () =>
  {
    let statenum:number = 0;
    let replaynum:number = 0;
    let cpnum:number = 0;
    const ui_state:ui.UI = new ui.UI(
      <HTMLDivElement>document.getElementById("ui")!,
      {
        load_state: (sn:number) => console.log("LOAD",sn),
        load_checkpoint: (sn:number) => console.log("LOADCP",sn),
        play_replay: (sn:number) => console.log("PLAY",sn),
        download_file: (category:"save"|"state"|"replay", file_name:string) => console.log("Save file",category,file_name)
      },false
    );
    (<HTMLAnchorElement>document.getElementById("save_button")!).addEventListener("click",
      () => ui_state.newSave("yet another save.srm"));
    (<HTMLAnchorElement>document.getElementById("start_replay_button")!).addEventListener("click",
      () => {
        ui_state.newReplay("yet another replay.replay"+replaynum.toString());
        replaynum +=1;
      });
    (<HTMLAnchorElement>document.getElementById("checkpoint_button")!).addEventListener("click",
      () => {
        ui_state.newCheckpoint("check"+cpnum.toString(),IMG_DATA);
        cpnum +=1;
      });
    (<HTMLAnchorElement>document.getElementById("save_state_button")!).addEventListener("click", () => {
      ui_state.newState("a state.state"+statenum.toString(), IMG_DATA);
      statenum += 1;
    });

    (<HTMLAnchorElement>document.getElementById("remove_last_save_button")!).addEventListener("click",
      () => ui_state.removeSave("yet another save.srm"));
    (<HTMLAnchorElement>document.getElementById("remove_last_replay_button")!).addEventListener("click",
      () => {
        replaynum -= 1;
        ui_state.removeReplay("yet another replay.replay"+replaynum.toString());
      });
    (<HTMLAnchorElement>document.getElementById("remove_last_checkpoint_button")!).addEventListener("click",
      () => {
        cpnum -= 1;
        ui_state.removeCheckpoint("check"+cpnum.toString());
      });
    (<HTMLAnchorElement>document.getElementById("remove_last_state_button")!).addEventListener("click", () => {
      statenum -= 1;
      ui_state.removeState("a state.state"+statenum.toString());
    });

    (<HTMLAnchorElement>document.getElementById("clear_ui_button")!).addEventListener("click",
      () => ui_state.clear());
    (<HTMLAnchorElement>document.getElementById("finish_replay_button")!).addEventListener("click",
      () => ui_state.clearCheckpoints());

    const emulator_window:HTMLDivElement = <HTMLDivElement>document.getElementById(UIIDConst.EMU_SINGLE_DIV);
    emulator_window.setAttribute("style", EMULATOR_WINDOW_STYLE);

  });
