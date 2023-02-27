interface UIController {
  load_state: (state_num:number) => void;
  download_file:(category:"save"|"state"|"movie", file_name:string) => void;
}

export class UI {
  state_elt:HTMLDivElement;
  saves_elt:HTMLDivElement;
  control:UIController;
  // ... functions go here
  constructor(state_elt:HTMLDivElement, saves_elt:HTMLDivElement, control:UIController) {
    this.state_elt = state_elt;
    this.saves_elt = saves_elt;
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
  clear() {
    this.state_elt.childNodes.forEach((elt:ChildNode,_key,_par) => {
      elt.remove();
    });
  }
}
