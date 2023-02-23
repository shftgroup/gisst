export class UI {
    state_elt;
    saves_elt;
    control;
    // ... functions go here
    constructor(state_elt, saves_elt, control) {
        this.state_elt = state_elt;
        this.saves_elt = saves_elt;
        this.control = control;
    }
    newSave(save_file) {
        console.log("found new save", save_file);
    }
    newState(state_file, state_thumbnail_b64_png) {
        const img = new Image();
        const img_data = "data:image/png;base64," + state_thumbnail_b64_png;
        img.src = img_data;
        const num_str = (state_file.match(/state([0-9]+)$/)?.[1]) ?? "0";
        const save_num = parseInt(num_str, 10);
        img.addEventListener("click", () => {
            console.log("Load", state_file, save_num);
            this.control["load_state"](save_num);
        });
        this.state_elt.appendChild(img);
    }
    clearStates() {
        this.state_elt.childNodes.forEach((elt, _key, _par) => {
            elt.remove();
        });
    }
}
