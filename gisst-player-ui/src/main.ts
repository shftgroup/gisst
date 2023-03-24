interface UIController {
  load_state: (state_num:number) => void;
  play_replay: (replay_num:number) => void;
  load_checkpoint: (state_num:number) => void;
  download_file:(category:"save"|"state"|"replay", file_name:string) => void;
}

export class UI {
  control:UIController;

  ui_root:HTMLDivElement;
  state_elt:HTMLOListElement;
  saves_elt:HTMLOListElement;
  replay_elt:HTMLOListElement;
  checkpoint_elt:HTMLOListElement;

  entries_by_name:Record<string,HTMLLIElement>;
  
  // ... functions go here
  constructor(ui_root:HTMLDivElement, control:UIController) {
    this.control = control;

    this.ui_root = ui_root;
    this.saves_elt = <HTMLOListElement>document.createElement("ol");
    ui_root.appendChild(this.saves_elt);
    this.state_elt = <HTMLOListElement>document.createElement("ol");
    ui_root.appendChild(this.state_elt);
    this.replay_elt = <HTMLOListElement>document.createElement("ol");
    ui_root.appendChild(this.replay_elt);
    this.checkpoint_elt = <HTMLOListElement>document.createElement("ol");
    ui_root.appendChild(this.checkpoint_elt);
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
    const img = new Image();
    const img_data = state_thumbnail.startsWith("data:image") ? state_thumbnail : "data:image/png;base64,"+state_thumbnail;
    img.src = img_data;
    const num_str = (state_file.match(/state([0-9]+)$/)?.[1]) ?? "0";
    const save_num = parseInt(num_str,10);
    img.addEventListener("click", () => {
      console.log("Load",state_file,save_num);
      this.control["load_state"](save_num);
    });
    const a = <HTMLAnchorElement>document.createElement("a");
    a.textContent=state_file;
    a.addEventListener("click", () => this.control.download_file("state",state_file));
    const li = <HTMLLIElement>document.createElement("li");
    li.appendChild(img);
    li.appendChild(a);
    this.state_elt.appendChild(li);
    this.entries_by_name["st__"+state_file] = li;
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
    let toRemove = [];
    for(let lit in this.entries_by_name) {
      if(lit.startsWith("cp__")) {
        this.entries_by_name[lit].remove();
        toRemove.push(lit);
      }
    }
    for(let lit of toRemove) {
      delete this.entries_by_name[lit];
    }
    this.checkpoint_elt.innerHTML = "";
  }
  newCheckpoint(check_name:string, state_thumbnail:string) {
    const img = new Image();
    const img_data = state_thumbnail.startsWith("data:image") ? state_thumbnail : "data:image/png;base64,"+state_thumbnail;
    img.src = img_data;
    const num_str = (check_name.match(/check([0-9]+)$/)?.[1]) ?? "0";
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
    for(let lit in this.entries_by_name) {
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
