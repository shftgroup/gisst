import { Replay, Evt } from './v86replay';
export interface StateInfo {
    name: string;
    thumbnail: string;
}
export interface EmbedV86Config {
    wasm_root: string;
    bios_root: string;
    content_root: string;
    container: HTMLDivElement;
    record_replay: (nom: string) => void;
    stop_replay: () => void;
    states_changed: (added: StateInfo[], removed: StateInfo[]) => void;
    replay_checkpoints_changed: (added: StateInfo[], removed: StateInfo[]) => void;
}
export declare class State {
    name: string;
    state: ArrayBuffer;
    thumbnail: string;
    constructor(name: string, state: ArrayBuffer, thumbnail: string);
}
export declare class EmbedV86 {
    emulator: V86Starter | null;
    config: EmbedV86Config;
    states: State[];
    replays: Replay[];
    active_replay: number | null;
    constructor(config: EmbedV86Config);
    clear(): void;
    get_active_replay(): Replay;
    save_state(): Promise<void>;
    record_replay(): Promise<void>;
    stop_replay(): Promise<void>;
    load_state_slot(n: number): Promise<void>;
    play_replay_slot(n: number): Promise<void>;
    download_file(category: "state" | "save" | "replay", file_name: string): [Blob, string];
    replay_log(evt: Evt, val: any): void;
    replay_tick(): void;
    run(content: string, entryState: boolean, movie: boolean): Promise<void>;
}