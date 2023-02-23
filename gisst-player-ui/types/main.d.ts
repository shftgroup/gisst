interface UIController {
    load_state: (state_num: number) => void;
}
export declare class UI {
    state_elt: HTMLDivElement;
    saves_elt: HTMLDivElement;
    control: UIController;
    constructor(state_elt: HTMLDivElement, saves_elt: HTMLDivElement, control: UIController);
    newSave(save_file: string): void;
    newState(state_file: string, state_thumbnail_b64_png: string): void;
    clear(): void;
}
export {};
