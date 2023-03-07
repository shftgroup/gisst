interface UIController {
  load_state: (state_num:number) => void;
  play_replay: (replay_num:number) => void;
  download_file:(category:"save"|"state"|"replay", file_name:string) => void;
}

export class UI {
  state_elt:HTMLDivElement;
  saves_elt:HTMLDivElement;
  replay_elt:HTMLDivElement;
  control:UIController;
  // ... functions go here
  constructor(state_elt:HTMLDivElement, replay_elt:HTMLDivElement, saves_elt:HTMLDivElement, control:UIController) {
    this.state_elt = state_elt;
    this.saves_elt = saves_elt;
    this.replay_elt = replay_elt;
    this.control = control;
  } 
  newSave(save_file:string) {
    console.log("found new save",save_file);
    const a = <HTMLAnchorElement>document.createElement("a");
    a.textContent=save_file;
    a.addEventListener("click", () => this.control.download_file("save",save_file));
    this.saves_elt.appendChild(a);
  }
  newState(state_file:string, state_thumbnail_b64_png:string) {
    const img = new Image();
    const img_data = "data:image/png;base64,"+state_thumbnail_b64_png;
    img.src = img_data;
    const num_str = (state_file.match(/state([0-9]+)$/)?.[1]) ?? "0";
    const save_num = parseInt(num_str,10);
    img.addEventListener("click", () => {
      console.log("Load",state_file,save_num);
      this.control["load_state"](save_num);
    });
    this.state_elt.appendChild(img);
    const a = <HTMLAnchorElement>document.createElement("a");
    a.textContent=state_file;
    a.addEventListener("click", () => this.control.download_file("state",state_file));
    this.state_elt.appendChild(a);
  }
  newReplay(replay_file:string) {
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
    this.replay_elt.appendChild(a_file);
  }
  clear() {
    this.state_elt.childNodes.forEach((elt:ChildNode,_key,_par) => {
      elt.remove();
    });
  }
}
