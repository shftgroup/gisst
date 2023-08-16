// Importing main scss file, vite will process and include bootstrap
import {DBRecord} from "./models";

export {UIIDConst} from "./template_consts"
import '../scss/styles.scss'
import * as bootstrap from 'bootstrap'

import templates from "../html/templates.html?raw"
import {UITemplateConst, UIIDConst } from "./template_consts"

interface UIController {
  load_state: (state_num:number) => void;
  play_replay: (replay_num:number) => void;
  load_checkpoint: (state_num:number) => void;
  download_file:(category:"save"|"state"|"replay", file_name:string) => void;
  upload_file:(category:"save"|"state"|"replay", file_name:string, metadata: DBRecord) => void;
}

export class UI {
  // static declarations for UI element names
  // assuming a single emulator window right now, will modify for multiple windows
  static readonly state_button_id = "state_button";
  static readonly save_button_id = "save_button";
  static readonly checkpoint_button_id = "checkpoint_button";
  static readonly replay_button_id = "start_replay_button";
  static readonly gisst_saves_list_content_id = "gisst-saves";
  static readonly gisst_states_list_content_id = "gisst-states";
  static readonly gisst_replays_list_content_id = "gisst-replays";
  static readonly gisst_checkpoints_list_content_id = "gisst-checkpoints";
  ui_date_format:Intl.DateTimeFormatOptions = {
    day: "2-digit", year: "numeric", month: "short",
    hour:"numeric", minute:"numeric", second:"numeric"
  }
  control:UIController;
  emulator_div:HTMLDivElement;

  headless:boolean;

  ui_root:HTMLDivElement;
  saves_elt:HTMLOListElement;
  replay_elt:HTMLOListElement;
  checkpoint_elt:HTMLOListElement;

  entries_by_name:Record<string,HTMLElement>;
  
  // ... functions go here
  constructor(ui_root:HTMLDivElement, control:UIController, headless:boolean) {
    let _unused = bootstrap.Alert; // needed to force TS compile to import bootstrap
    this.ui_root = ui_root;
    this.control = control;
    this.headless = headless;

    // Make sure the ui root div has a 'container' bootstrap class
    if (!this.ui_root.classList.contains("container")){
      this.ui_root.classList.add("container");
    }

    this.emulator_div = <HTMLDivElement>elementFromTemplates(UITemplateConst.EMULATOR_SINGLE_DIV);

    // Configure initial UI state
    // May turn this into a separate set of functions?
    if (this.headless) {
      const ui_headless_grid = <HTMLDivElement>elementFromTemplates(UITemplateConst.GRID_CONTAINER_HEADLESS);
      this.ui_root.appendChild(ui_headless_grid);
    } else {
      const ui_embedded_grid = <HTMLDivElement>elementFromTemplates(UITemplateConst.GRID_CONTAINER_EMBEDDED);

      // Attach emulator div to ui root
      ui_embedded_grid.querySelector("#"+UIIDConst.EMU_EMBEDDED_COL)!
          .appendChild(this.emulator_div);

      // Create emulator control bar
      ui_embedded_grid.querySelector("#"+UIIDConst.EMU_CONTROL_BAR_COL)!
          .appendChild(elementFromTemplates(UITemplateConst.EMULATOR_CONTROL_BAR));

      // Add object information tabs
      ui_embedded_grid.querySelector("#"+UIIDConst.EMU_CONTEXT_DIV)!
          .appendChild(elementFromTemplates(UITemplateConst.EMULATOR_OBJECTS_TABS_EMBEDDED));
      this.ui_root.appendChild(ui_embedded_grid);
    }
    // Configure emulator manipulation toolbar
    this.saves_elt = <HTMLOListElement>document.createElement("ol");
    this.ui_root.appendChild(this.saves_elt);
    this.replay_elt = <HTMLOListElement>document.createElement("ol");
    this.ui_root.appendChild(this.replay_elt);
    this.checkpoint_elt = <HTMLOListElement>document.createElement("ol");
    this.ui_root.appendChild(this.checkpoint_elt);
    this.entries_by_name = {};
  }

  newSave(save_file:string) {
    console.log("found new save",save_file);
    const a = <HTMLAnchorElement>document.createElement("a");
    a.textContent=save_file;
    a.addEventListener("click", () => this.control.download_file("save",save_file));
    const li = <HTMLLIElement>document.createElement("li");
    li.appendChild(a);
    this.saves_elt.appendChild(li);
    this.entries_by_name["sv__"+save_file] = li;
  }
  newState(state_file:string, state_thumbnail:string) {
    console.log("found new state", state_file);
    // Create state list template object
    const new_state_list_object = <HTMLDivElement>elementFromTemplates("state_list_object");

    // Add img data to state_list_object and create on click load
    // state from img
    const img = <HTMLImageElement>new_state_list_object.querySelector("img");
    const img_data = state_thumbnail.startsWith("data:image") ? state_thumbnail : "data:image/png;base64,"+state_thumbnail;
    img.src = img_data;

    const num_str = (state_file.match(/state([0-9]+)$/)?.[1]) ?? "0";
    const save_num = parseInt(num_str,10);
    img.addEventListener("click", () => {
      console.log("Load",state_file,save_num);
      this.control["load_state"](save_num);
    });

    // Add state descriptive information and download link
    const a = <HTMLAnchorElement>document.createElement("a");
    a.textContent=state_file;
    a.addEventListener("click", () => this.control.download_file("state",state_file));
    const state_object_title = <HTMLHeadElement>new_state_list_object.querySelector("h5");
    state_object_title.appendChild(a);
    const state_object_timestamp = <HTMLElement>new_state_list_object.querySelector("small");
    state_object_timestamp.textContent = `Created ${new Date().toLocaleDateString("en-US", this.ui_date_format)}`;

    const gisst_state_tab = <HTMLDivElement>document.getElementById(UI.gisst_states_list_content_id);
    gisst_state_tab.appendChild(new_state_list_object);

    this.entries_by_name["st__"+state_file] = new_state_list_object;
  }
  newReplay(replay_file:string) {
    this.clearCheckpoints();
    const num_str = (replay_file.match(/replay([0-9]+)$/)?.[1]) ?? "0";
    const replay_num = parseInt(num_str,10);
    const a_replay = <HTMLAnchorElement>document.createElement("a");
    a_replay.textContent="Click to Play:";
    a_replay.addEventListener("click", () => {
      console.log("Play",replay_file,replay_num);
      this.control["play_replay"](replay_num);
    });
    this.replay_elt.appendChild(a_replay);
    const a_file = <HTMLAnchorElement>document.createElement("a");
    a_file.textContent=replay_file;
    a_file.addEventListener("click", () => this.control.download_file("replay",replay_file));
    const li = <HTMLLIElement>document.createElement("li");
    li.appendChild(a_replay);
    li.appendChild(a_file);
    this.replay_elt.appendChild(li);
    this.entries_by_name["rp__"+replay_file] = li;
  }
  clearCheckpoints() {
    const toRemove = [];
    for(const lit in this.entries_by_name) {
      if(lit.startsWith("cp__")) {
        this.entries_by_name[lit].remove();
        toRemove.push(lit);
      }
    }
    for(const lit of toRemove) {
      delete this.entries_by_name[lit];
    }
    this.checkpoint_elt.innerHTML = "";
  }
  newCheckpoint(check_name:string, state_thumbnail:string) {
    const img = new Image();
    const img_data = state_thumbnail.startsWith("data:image") ? state_thumbnail : "data:image/png;base64,"+state_thumbnail;
    img.src = img_data;
    const num_str = (check_name.match(/(check|state)([0-9]+)$/)?.[2]) ?? "0";
    const save_num = parseInt(num_str,10);
    img.addEventListener("click", () => {
      console.log("Load CP",check_name,save_num);
      this.control["load_checkpoint"](save_num);
    });
    const li = <HTMLLIElement>document.createElement("li");
    li.appendChild(img);
    this.checkpoint_elt.appendChild(li);
    this.entries_by_name["cp__"+check_name] = li;
  }
  clear() {
    for(const lit in this.entries_by_name) {
      this.entries_by_name[lit].remove();
    }
    this.entries_by_name = {};
  }
  removeLit(fq_name:string) {
    if(fq_name in this.entries_by_name) {
      this.entries_by_name[fq_name].remove();
      delete this.entries_by_name[fq_name];
    }
  }
  removeState(state_file:string) {
    this.removeLit("st__"+state_file);
  }
  removeReplay(replay_file:string) {
    this.removeLit("rp__"+replay_file);
  }
  removeSave(save_file:string) {
    this.removeLit("sv__"+save_file);
  }
  removeCheckpoint(cp_file:string) {
    this.removeLit("cp__"+cp_file);
  }
}

function elementFromTemplates(template_name: string): Node {
  const templates_element = <HTMLTemplateElement>document.createElement("template");
  templates_element.innerHTML = templates.trim();
  const template_element = <HTMLTemplateElement>templates_element.content.querySelector("#" + template_name)!;
  return template_element.content.cloneNode(true);
}
