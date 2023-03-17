import { Replay, Evt } from './v86replay';
export interface EmbedV86Config {
    wasm_root: string;
    bios_root: string;
    content_root: string;
    container: HTMLDivElement;
}
export declare class EmbedV86 {
    emulator: V86Starter | null;
    config: EmbedV86Config;
    states: ArrayBuffer[];
    replays: Replay[];
    active_replay: number | null;
    constructor(config: EmbedV86Config);
    clear(): void;
    save_state(callback: (nom: string, screen: string) => void): Promise<void>;
    record_replay(callback: (nom: string) => void): Promise<void>;
    stop_replay(): Promise<void>;
    load_state_slot(n: number): Promise<void>;
    play_replay_slot(n: number): Promise<void>;
    download_file(category: "state" | "save" | "replay", file_name: string): [Blob, string];
    replay_log(evt: Evt, val: any): void;
    replay_tick(): void;
    run(content: string, entryState: boolean, movie: boolean): Promise<void>;
}
