import * as ui from './main';

const IMG_DATA:string = "iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAYAAADDPmHLAAABbWlDQ1BpY2MAACiRdZG7S8NQFMZ/poriA0EdRBwyVBFUEAVx1Dp0KSJVwdeSpmkr9BGSFCmugouD4CC6+Br8D3QVXBUEQRFE3Nx9LVLiuUaoiL3h5vz47v0OJ19Ai2XNnFs7Drm858SjEX1+YVGvf0ajkzb6CRuma09MT8eouj5uqVH1ZlD1qn7v39WUtFwTahqER03b8YRlGmKrnq14U7jDzBhJ4QPhAUcGFL5UeiLgJ8XpgN8UO7PxSdBUTz39ixO/2Mw4OeE+4XAuWzR/5lFf0mzl52akdsnuxiVOlAg6CYqskMVjUGpeMvvfN/Ttm6IgHlPeNiUccaTJiHdA1KJ0taSmRLfkyVJSuf/N002NDAfdmyNQ9+j7rz1Qvw3lLd//PPT98hGEHuA8X/EXJKexd9G3Klp4H1rX4fSioiV24GwDOu9twzG+pZBsLZWClxNoWYD2a2hcCrL6Oef4DmbX5Bddwe4e9Mr91uUvPjJoJucWBkIAAAAJcEhZcwAACxIAAAsSAdLdfvwAAAERSURBVHja7dZBCsAgDADBGPz/k2P7gp6aQ3AGehVJV3RFxAmulUYgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACYLjdsWhVmeyfpzRzVgDdm75J92Hyl7wBEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAB82F0LV5XpDrDe7xiDKwABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAEgAASAABAAAkAAzPMAGvkL/eKCZR0AAAAASUVORK5CYII=";

addEventListener("load", () =>
  {
    let statenum:number = 0;
    let replaynum:number = 0;
    let ui_state:ui.UI = new ui.UI(
      <HTMLDivElement>document.getElementById("states")!,
      <HTMLDivElement>document.getElementById("replays")!,
      <HTMLDivElement>document.getElementById("saves")!,
      {
        load_state: (sn:number) => console.log("LOAD",sn),
        play_replay: (sn:number) => console.log("PLAY",sn),
        download_file: (category:"save"|"state"|"replay", file_name:string) => console.log("Save file",category,file_name)
      }
    );
    (<HTMLAnchorElement>document.getElementById("new_save_button")!).addEventListener("click",
      () => ui_state.newSave("yet another save.srm"));
    (<HTMLAnchorElement>document.getElementById("new_replay_button")!).addEventListener("click",
      () => {
        replaynum +=1;
        ui_state.newReplay("yet another replay.replay"+replaynum.toString());
      });
    (<HTMLAnchorElement>document.getElementById("new_state_button")!).addEventListener("click", () => {
      statenum += 1;
      ui_state.newState("a state.state"+statenum.toString(), IMG_DATA);
    });
  });
