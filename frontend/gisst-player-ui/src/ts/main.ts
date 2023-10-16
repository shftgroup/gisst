// Importing main scss file, vite will process and include bootstrap
import {
  FrontendConfig,
  generateStateFields,
    generateReplayFields,
  State,
    Replay,
  Metadata,
  ReplayFileLink,
  StateFileLink,
  canEdit
} from "./models";

import '../scss/styles.scss'
import * as bootstrap from 'bootstrap'
// import * as uuid from 'uuid'

import templates from "../html/templates.html?raw"
import {UITemplateConst, UIIDConst } from "./template_consts"

export enum ReplayMode {
  Inactive=0,
  Record,
  Playback,
  Finished,
}

interface UIController {
  toggle_mute: () => void;
  load_state: (state_num:number) => void;
  save_state: () => void;
  play_replay: (replay_num:number) => void;
  start_replay:() => void;
  stop_and_save_replay:() => void;
  load_checkpoint: (state_num:number) => void;
  download_file:(category:"save"|"state"|"replay", file_name:string) => void;
  upload_file:(category:"save"|"state"|"replay", file_name:string, metadata: Metadata) => Promise<Metadata>;
}

export class UI {
  // static declarations for UI element names
  // assuming a single emulator window right now, will modify for multiple windows
  static readonly gisst_saves_list_content_id = "gisst-saves";
  static readonly gisst_states_list_content_id = "gisst-states-list";
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
  replay_elt:HTMLUListElement;
  checkpoint_elt:HTMLDivElement;

  entries_by_name:Record<string,HTMLElement>;
  metadata_by_name:Record<string,Metadata>;
  current_config:FrontendConfig;

  // ... functions go here
  constructor(ui_root:HTMLDivElement, control:UIController, headless:boolean, config:FrontendConfig) {
    const _unused = bootstrap.Alert; // needed to force TS compile to import bootstrap
    if(_unused) {
      // needed to avoid TS compile issue
    }
    this.ui_root = ui_root;
    this.control = control;
    this.headless = headless;
    this.current_config = config;

    // Make sure the ui root div has a 'container' bootstrap class
    if (!this.ui_root.classList.contains("container")){
      this.ui_root.classList.add("container-fluid");
    }

    const emulator_div = <HTMLDivElement>elementFromTemplates(UITemplateConst.EMULATOR_SINGLE_DIV);

    // Configure initial UI state
    // May turn this into a separate set of functions?
    if (this.headless) {
      const ui_headless_grid = <HTMLDivElement>elementFromTemplates(UITemplateConst.GRID_CONTAINER_HEADLESS);
      this.ui_root.appendChild(ui_headless_grid);
    } else {
      const gisst_header = <HTMLElement>elementFromTemplates(UITemplateConst.GISST_PAGE_DARK_HEADER);
      const current_work = <HTMLSpanElement>document.createElement("span");
      current_work.innerHTML = "Playing with " + this.current_config.work.work_name;
      document.body.prepend(gisst_header);

      const ui_embedded_grid = <HTMLDivElement>elementFromTemplates(UITemplateConst.GRID_CONTAINER_EMBEDDED);

      // Attach emulator div to ui root
      ui_embedded_grid.querySelector("#"+UIIDConst.EMU_EMBEDDED_COL)!
        .appendChild(emulator_div);

      // Create emulator control bar
      ui_embedded_grid.querySelector("#"+UIIDConst.EMU_CONTROL_BAR_COL)!
          .appendChild(elementFromTemplates(UITemplateConst.EMULATOR_CONTROL_BAR));

      // Add object information tabs
      ui_embedded_grid.querySelector("#"+UIIDConst.EMU_CONTEXT_DIV)!
          .appendChild(elementFromTemplates(UITemplateConst.EMULATOR_OBJECTS_TABS_EMBEDDED));
      this.ui_root.appendChild(ui_embedded_grid);
      
      this.ui_root.querySelector("#"+UIIDConst.EMU_TOGGLE_MUTE_BUTTON)!.addEventListener("click", this.control.toggle_mute);
      this.ui_root.querySelector("#"+UIIDConst.EMU_SAVE_STATE_BUTTON)!.addEventListener("click", this.control.save_state);
      this.ui_root.querySelector("#"+UIIDConst.EMU_START_REPLAY_BUTTON)!.addEventListener("click", this.control.start_replay);
      this.ui_root.querySelector("#"+UIIDConst.EMU_FINISH_REPLAY_BUTTON)!.addEventListener("click", this.control.stop_and_save_replay);
    }
    // TODO do the right thing if this is headless
    this.emulator_div = <HTMLDivElement>document.getElementById("emulator_single_div")!;

    // Configure emulator manipulation toolbar
    this.saves_elt = <HTMLOListElement>document.createElement("ol");
    this.ui_root.appendChild(this.saves_elt);
    this.replay_elt = <HTMLUListElement>document.getElementById("gisst-replays-list");
    this.checkpoint_elt = <HTMLDivElement>document.getElementById("gisst-checkpoints-list");
    this.entries_by_name = {};
    this.metadata_by_name = {};
  }

  setReplayMode(mode:ReplayMode) {
    // TODO do something different if this.headless
    switch (mode) {
      case ReplayMode.Inactive:
      this.emulator_div.classList.remove("emulator-recording");
      this.emulator_div.classList.remove("emulator-playback");
      break;
      case ReplayMode.Record:
      this.emulator_div.classList.add("emulator-recording");
      this.emulator_div.classList.remove("emulator-playback");
      break;
      case ReplayMode.Playback:
      this.emulator_div.classList.remove("emulator-recording");
      this.emulator_div.classList.add("emulator-playback");
      break;
      case ReplayMode.Finished:
      // do nothing
      break;
    }
  }

  setConfig(config:FrontendConfig) {
    this.current_config = config;
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
  newState(state_file:string, state_thumbnail:string, metadata?:StateFileLink) {
    console.log("found new state", state_file, metadata);
    // Create state list template object
    const new_state_list_object = <HTMLDivElement>elementFromTemplates("card_list_object");
    new_state_list_object.querySelector(".card-list-object")!
        .setAttribute("id", valid_for_css(state_file));

    // Add img data to state_list_object and create on click load
    // state from img
    const img = <HTMLImageElement>new_state_list_object.querySelector("img");
    img.src = state_thumbnail.startsWith("data:image") ? state_thumbnail : "data:image/png;base64,"+state_thumbnail;

    const num_str = (state_file.match(/state([0-9]+)$/)?.[1]) ?? "0";
    const save_num = parseInt(num_str,10);
    img.addEventListener("click", () => {
      console.log("Load",state_file,save_num);
      this.control["load_state"](save_num);
    });

    const state_object_title = <HTMLHeadElement>new_state_list_object.querySelector("h5");
    state_object_title.textContent = state_file;
    const state_object_description = <HTMLElement>new_state_list_object.querySelector(".card-text");
    state_object_description.textContent = state_file;
    const state_object_timestamp = <HTMLElement>new_state_list_object.querySelector("small");
    state_object_timestamp.textContent = `Created ${new Date().toLocaleDateString("en-US", this.ui_date_format)}`;

    new_state_list_object.querySelector(".download-button")!.addEventListener("click", () => this.control.download_file("state", state_file));

    new_state_list_object.querySelector(".upload-button")!.addEventListener("click", () => {
      const metadata = this.metadata_by_name["st__"+state_file];
      if(!metadata.editing && !metadata.stored_on_server){
        this.control.upload_file("state", state_file, metadata)
            .then((md:Metadata) =>{
              this.completeUpload("st__"+state_file, md);
            });

      }
    });

    new_state_list_object.querySelector(".edit-button")!.addEventListener("click", (_e:Event) => {
      this.toggleEditState(state_file);
    })

    const gisst_state_tab = <HTMLDivElement>document.getElementById(UI.gisst_states_list_content_id);
    gisst_state_tab.appendChild(new_state_list_object);

    this.entries_by_name["st__"+state_file] = document.getElementById(valid_for_css(state_file))!;
    nonnull(this.current_config);
    const state_metadata:Metadata = {
      record: {
        state_id: metadata?.state_id || "DECAFBADDECAFBADDECAFBADDECAFBAD",
        instance_id: this.current_config.instance.instance_id,
        is_checkpoint: metadata?.is_checkpoint || false,
        file_id: metadata?.file_id || "DECAFBADDECAFBADDECAFBADDECAFBAD",
        state_description: metadata?.state_description || state_file,
        state_name: metadata?.state_name || state_file,
        state_derived_from: metadata?.state_derived_from || (this.current_config.start.type === "state" ? (this.current_config.start.data! as StateFileLink).state_id : null),
        screenshot_id: metadata?.screenshot_id || "DECAFBADDECAFBADDECAFBADDECAFBAD",
        replay_id: metadata?.replay_id || null,
        creator_id: metadata?.creator_id || "00000000-0000-0000-0000-000000000000",
        created_on: metadata?.created_on || new Date()
      },
      screenshot: state_thumbnail,
      stored_on_server: metadata !== undefined,
      editing: false
    }

    this.metadata_by_name["st__"+state_file] = state_metadata;
  }
  newReplay(replay_file:string, metadata?:ReplayFileLink) {
    this.clearCheckpoints();
    const num_str = (replay_file.match(/replay([0-9]+)$/)?.[1]) ?? "0";
    const replay_num = parseInt(num_str,10);

    const li = <HTMLUListElement>elementFromTemplates("replay_list_item");
    li.querySelector(".replay-list-item")!.setAttribute("id", valid_for_css(replay_file));
    li.querySelector(".replay-list-item-replay-name")!.textContent = replay_file;
    li.querySelector(".replay-list-item-replay-desc")!.textContent = replay_file;

    li.querySelector(".replay-list-item-play-button")!.addEventListener("click", () => {
      console.log("Play", replay_file, replay_num);
      this.control.play_replay(replay_num);
    });

    li.querySelector(".replay-list-item-download-button")!.addEventListener("click", () => {
      this.control.download_file("replay", replay_file);
    });

    li.querySelector(".replay-list-item-upload-button")!.addEventListener("click", () => {
      const metadata = this.metadata_by_name["rp__"+replay_file];
      this.control.upload_file("replay", replay_file, metadata)
          .then((md:Metadata) => {
            this.completeUpload("rp__"+replay_file, md);
          })
    });

    li.querySelector(".replay-list-item-edit-button")!.addEventListener("click", () => {
      this.toggleEditReplay(replay_file);
    });

    const replay_metadata:Metadata = {
      record: {
        replay_id: metadata?.replay_id || "DECAFBADDECAFBADDECAFBADDECAFBAD",
        replay_name: metadata?.replay_name || replay_file,
        replay_description: metadata?.replay_description || replay_file,
        instance_id: this.current_config.instance.instance_id,
        creator_id: metadata?.creator_id || "00000000-0000-0000-0000-000000000000",
        replay_forked_from: metadata?.replay_forked_from || (this.current_config.start.type === "replay" ? (this.current_config.start.data! as ReplayFileLink).replay_id : null),
        file_id: metadata?.file_id || "DECAFBADDECAFBADDECAFBADDECAFBAD",
        created_on: metadata?.created_on || new Date()
      },
      stored_on_server: metadata !== undefined,
      editing: false,
      screenshot: ""
    }

    this.replay_elt.appendChild(li);
    this.entries_by_name["rp__"+replay_file] = document.getElementById(valid_for_css(replay_file))!;
    this.metadata_by_name["rp__"+replay_file] = replay_metadata;
  }
  clearCheckpoints() {
    const toRemove = [];
    for(const lit in this.entries_by_name) {
      if(lit.startsWith("cp__")) {
        toRemove.push(lit);
      }
    }
    for(const lit of toRemove) {
      this.removeLit(lit);
    }
    this.checkpoint_elt.innerHTML = "";
  }
  newCheckpoint(check_name:string, state_thumbnail:string) {
    const checkpoint_list_object = <HTMLDivElement>elementFromTemplates("card_list_object");
    checkpoint_list_object.querySelector(".card-list-object")!.setAttribute("id", valid_for_css(check_name));

    const img = <HTMLImageElement>checkpoint_list_object.querySelector("img")!;
    img.src = state_thumbnail.startsWith("data:image") ? state_thumbnail : "data:image/png;base64,"+state_thumbnail;

    const num_str = (check_name.match(/(check|state)([0-9]+)$/)?.[2]) ?? "0";
    const save_num = parseInt(num_str,10);

    img.addEventListener("click", () => {
      console.log("Load CP",check_name,save_num);
      this.control["load_checkpoint"](save_num);
    });

    checkpoint_list_object.querySelector(".upload-button")!.remove();

    this.checkpoint_elt.appendChild(checkpoint_list_object);
    this.entries_by_name["cp__"+check_name] = document.getElementById(valid_for_css(check_name))!;
    console.log("added CP","cp__"+check_name,this.entries_by_name["cp__"+check_name]);
  }
  clear() {
    for(const lit in this.entries_by_name) {
      this.entries_by_name[lit].remove();
      delete this.metadata_by_name[lit]
    }

    this.entries_by_name = {};
    this.metadata_by_name = {};
  }
  removeLit(fq_name:string) {
    if(fq_name in this.entries_by_name) {
      this.entries_by_name[fq_name].remove();
      delete this.entries_by_name[fq_name];
      delete this.metadata_by_name[fq_name];
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

  toggleEditReplay(replay_file:string) {
    const replay_fields = generateReplayFields();
    const replay_list_object = <HTMLDivElement>this.entries_by_name["rp__"+replay_file];
    const replay_metadata = this.metadata_by_name["rp__"+replay_file];

    if(replay_metadata.stored_on_server){
      return;
    }
    if(!replay_metadata.editing) {
      replay_metadata.editing = true;

      for (const field of replay_fields) {
        if(canEdit("replay", field.field_name)){
          const field_element:HTMLParagraphElement = document.createElement("p");
          field_element.classList.add(valid_for_css(replay_file) + "-edit-fields");
          field_element.classList.add("d-flex");
          field_element.classList.add("flex-row");
          const ele_id = valid_for_css(replay_file) + "_" + field.field_name;
          if (field.value_type === "string"){
            field_element.innerHTML = `<label for="${ele_id}">${field.field_name}</label><input type="text" class="${valid_for_css(replay_file)}-field" id="${ele_id}" name="${ele_id}"/>`;
            const input_element:HTMLInputElement = field_element.querySelector("#"+ele_id)!;
            input_element.value = <string>(replay_metadata.record as Replay)[field.field_name as keyof Replay];
            input_element.addEventListener("change", (e:Event) => {
              (replay_metadata.record as Replay)[field.field_name] = (e.currentTarget! as HTMLInputElement).value;
            });
          }
          replay_list_object.querySelector(".card-body")!.appendChild(field_element)
        }
      }
    } else {
      replay_metadata.editing = false;
      replay_list_object.querySelector(".card-title")!.textContent = (replay_metadata.record as Replay).replay_name;
      replay_list_object.querySelector(".card-text")!.textContent = (replay_metadata.record as Replay).replay_description;
      const edit_fields = replay_list_object.querySelectorAll("." + valid_for_css(replay_file) + "-edit-fields")!;
      for(let i = 0; i < edit_fields.length; i++){
        edit_fields[i].remove();
      }
    }
  }

  toggleEditState(state_file:string) {
    const state_fields = generateStateFields();
    const state_list_object = <HTMLDivElement>this.entries_by_name["st__"+state_file];
    const state_metadata = this.metadata_by_name["st__"+state_file];

    if(state_metadata.stored_on_server){
      return;
    }
    if(!state_metadata.editing) {
      state_metadata.editing = true;

      for (const field of state_fields) {
        if(canEdit("state", field.field_name)){
          const field_element:HTMLParagraphElement = document.createElement("p");
          field_element.classList.add(valid_for_css(state_file) + "-edit-fields");
          const ele_id = valid_for_css(state_file) + "_" + field.field_name;
          if (field.value_type === "string"){
            field_element.innerHTML = `<label for="${ele_id}">${field.field_name}</label><input type="text" class="${valid_for_css(state_file)}-field" id="${ele_id}" name="${ele_id}"/>`;
            const input_element:HTMLInputElement = field_element.querySelector("#"+ele_id)!;
            input_element.value = <string>(state_metadata.record as State)[field.field_name as keyof State];
            input_element.addEventListener("change", (e:Event) => {
              (state_metadata.record as State)[field.field_name] = (e.currentTarget! as HTMLInputElement).value;
            });
          } else if (field.value_type === "boolean") {
            field_element.innerHTML = `<input type="checkbox" class="${valid_for_css(state_file)}-field" id="${ele_id}" name="${ele_id}"/><label for="${ele_id}">${field.field_name.toUpperCase()}</label>`;
            const input_element:HTMLInputElement = field_element.querySelector("#"+ele_id)!;
            input_element.checked = <boolean>(state_metadata.record as State)[field.field_name as keyof State];
            input_element.addEventListener("change", (e:Event) => {
              (state_metadata.record as State)[field.field_name] = (e.currentTarget! as HTMLInputElement).checked;
            });
          }
          state_list_object.appendChild(field_element)
        }
      }
    } else {
      state_metadata.editing = false;
      state_list_object.querySelector("h5")!.textContent = (state_metadata.record as State).state_name;
      state_list_object.querySelector(".card-text")!.textContent = (state_metadata.record as State).state_description;
      const edit_fields = state_list_object.querySelectorAll("." + valid_for_css(state_file) + "-edit-fields")!;
      for(let i = 0; i < edit_fields.length; i++){
        edit_fields[i].remove();
      }
    }
  }
  completeUpload(item_name:string, md:Metadata){
    const item_object = this.entries_by_name[item_name];
    md.stored_on_server = true;
    delete this.metadata_by_name[item_name];
    this.metadata_by_name[item_name] = md;
    // Change the upload icon to filled
    item_object.querySelector(".bi-cloud-upload")!.classList.add("hidden");
    item_object.querySelector(".bi-cloud-arrow-up-fill")!.classList.remove("hidden");
  }
}

function elementFromTemplates(template_name: string): Node {
  const templates_element = <HTMLTemplateElement>document.createElement("template");
  templates_element.innerHTML = templates.trim();
  const template_element = <HTMLTemplateElement>templates_element.content.querySelector("#" + template_name)!;
  return template_element.content.cloneNode(true);
}

function valid_for_css(s:string): string {
  return s.replace(/[^_a-zA-Z0-9-]/g, "_")
}

function nonnull(obj:number|object|null):asserts obj {
  if(obj == null) {
    throw "Must be non-null";
  }
}


export {UIIDConst} from "./template_consts"
export {GISSTDBConnector} from "./db"
export * as GISSTModels from "./models"
