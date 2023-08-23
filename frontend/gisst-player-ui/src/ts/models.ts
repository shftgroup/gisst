export type DBRecord = Environment | Work | Save | Replay | State | Instance | Screenshot;
export type DBFileRecord = Save | State | Replay;

export interface Metadata {
    record: DBFileRecord,
    screenshot: string,
    stored_on_server: boolean
}

export enum ObjectRole {
    Content = "content",
    Dependency = "dependency",
    Config = "config"
}

export interface PlayerStartTemplateInfo {
    type: "cold" | "state" | "replay",
    data?: StateFileLink | ReplayFileLink
}

export interface FrontendConfig {
    environment: Environment,
    instance: Instance,
    manifest: ObjectFileLink[],
    save: Save,
    start: PlayerStartTemplateInfo
}

export interface ObjectFileLink {
    file_dest_path: string,
    file_filename: string,
    file_hash: string,
    file_source_path: string,
    object_id: string,
    object_role: ObjectRole
}

export interface StateFileLink {
    state_id: string,
    instance_id: string,
    is_checkpoint: boolean,
    state_name: string,
    state_description: string,
    screenshot_id?: string,
    replay_id?: string,
    creator_id?: string,
    state_replay_index?: number,
    state_derived_from?: string,
    created_on?: Date,
    file_hash: string,
    file_filename: string,
    file_source_path: string,
    file_dest_path: string
}

export interface ReplayFileLink {
    replay_id: string,
    instance_id: string,
    creator_id: string,
    replay_forked_from?: string,
    created_on?: Date,
    file_hash: string,
    file_filename: string,
    file_source_path: string,
    file_dest_path: string

}
export interface Environment {
    environment_id: string,
    environment_name: string,
    environment_framework: string,
    environment_core_name: string,
    environment_core_version: string,
    environment_derived_from: string,
    environment_config: JSON,
}

export interface Work {
    work_id: string,
    work_name: string,
    work_version: string,
    work_platform: string,
    created_on: Date
}

export interface Save {
    save_id: string,
    instance_id: string,
    save_short_desc: string,
    save_description: string,
    file_id: string,
    creator_id: string,
    created_on: Date
}
export interface State {
    state_id: string,
    instance_id: string,
    is_checkpoint: boolean,
    file_id: string,
    state_name: string,
    state_description: string,
    screenshot_id: string,
    replay_id: string,
    creator_id: string,
    state_derived_from: string,
    created_on: Date
}

export interface Replay {
    replay_id: string,
    instance_id: string,
    creator_id: string,
    replay_forked_from: string,
    file_id: string,
    created_on: Date
}
export interface Instance {
    instance_id: string,
    environment_id: string,
    work_id: string,
    instance_config: JSON,
    created_on: Date
}

export interface Screenshot {
    screenshot_id: string,
    screenshot_data: string
}

export interface FullInstance {
    info: Instance,
    work: Work,
    states: State[],
    replays: Replay[],
    saves: Save[]
}