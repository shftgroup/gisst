export declare enum Evt {
    Checkpoint = 0,
    KeyCode = 1,
    MouseClick = 2,
    MouseDelta = 3,
    MouseAbsolute = 4,
    MouseWheel = 5
}
export declare const EvtNames: (string | null)[];
declare enum ReplayMode {
    Inactive = 0,
    Record = 1,
    Playback = 2,
    Finished = 3
}
export declare class Replay {
    events: ReplayEvent[];
    index: number;
    id: string;
    mode: ReplayMode;
    last_time: number;
    wraps: number;
    private constructor();
    replay_time(insn_counter: number): bigint;
    log_evt(emulator: V86Starter, code: Evt, val: any): void;
    tick(emulator: V86Starter): Promise<void>;
    static start_recording(emulator: V86Starter): Promise<Replay>;
    private finish_playback;
    private finish_recording;
    stop(emulator: V86Starter): Promise<void>;
    start_playback(emulator: V86Starter): Promise<void>;
}
declare class ReplayEvent {
    when: bigint;
    code: Evt;
    value: any;
    constructor(when: bigint, code: Evt, value: any);
}
export {};
