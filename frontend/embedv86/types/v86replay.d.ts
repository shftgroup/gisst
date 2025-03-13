export declare enum Evt {
    KeyCode = 0,
    MouseClick = 1,
    MouseDelta = 2,
    MouseAbsolute = 3,
    MouseWheel = 4
}
export declare const EvtNames: string[];
export declare enum ReplayMode {
    Inactive = 0,
    Record = 1,
    Playback = 2,
    Finished = 3
}
export declare class Checkpoint {
    state: ArrayBuffer;
    name: string;
    thumbnail: string;
    when: number;
    event_index: number;
    constructor(when: number, name: string, event_index: number, state: ArrayBuffer, thumbnail: string);
}
export declare class Replay {
    events: ReplayEvent[];
    checkpoints: Checkpoint[];
    index: number;
    checkpoint_index: number;
    id: string;
    mode: ReplayMode;
    last_time: number;
    wraps: number;
    private constructor();
    reset_to_checkpoint(n: number, mode: ReplayMode, emulator: V86): Checkpoint[];
    private seek_internal;
    private resume;
    current_time(): number;
    replay_time(insn_counter: number): number;
    cpu_time(t: number): [number, number];
    log_evt(emulator: V86, code: Evt, val: object | number): void;
    make_checkpoint(emulator: V86): Promise<void>;
    tick(emulator: V86): Promise<void>;
    static start_recording(emulator: V86): Promise<Replay>;
    private finish_playback;
    private finish_recording;
    stop(emulator: V86): Promise<void>;
    start_playback(emulator: V86): Promise<void>;
    serialize(): Promise<ArrayBuffer>;
    static deserialize(buf: ArrayBuffer): Promise<Replay>;
}
export declare class ReplayEvent {
    when: number;
    code: Evt;
    value: object | number;
    constructor(when: number, code: Evt, value: object | number);
}
