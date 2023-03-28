export interface ReplayInfo {
    id: string;
    data: Uint8Array;
}
export declare function replay_info(raw_data: Uint8Array): ReplayInfo;
export declare function replay_of_state(raw_bytes: Uint8Array): ReplayInfo | null;
