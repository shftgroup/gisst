export interface Environment {
    environment_id: string,
    environment_name: string,
    environment_framework: string,
    environment_core_name: string,
    environment_core_version: string,
    environment_derived_from: string,
    environment_config: JSON
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

export interface FullInstance {
    info: Instance,
    work: Work,
    states: State[],
    replays: Replay[],
    saves: Save[]
}