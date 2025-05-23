export type DBRecord = Environment | Work | Save | Replay | State | Instance | ScreenshotCreateData | Screenshot;
export type DBFileRecord = Save | State | Replay;

export type DBField = {
    field_name: string,
    value_type: string,
    editable: boolean
}

export function canEdit(record_type:string, record_field_name: string):boolean {
     let fields:DBField[];
     if (record_type === "state") {
         fields = generateStateFields();
     } else if (record_type === "replay") {
         fields = generateReplayFields();
     } else if (record_type === "save") {
         fields = generateSaveFields();
     }
     for (const field of fields!){
         if(field.field_name === record_field_name) {
             return field.editable
         }
     }

     return false;
}

export interface Metadata {
    record: DBFileRecord,
    screenshot: string,
    stored_on_server: boolean,
    editing: boolean,
    group_key?: string
}

export enum ObjectRole {
    Content = "content",
    Dependency = "dependency",
    Config = "config"
}

export interface PlayerStartTemplateInfo {
    type: "cold" | "state" | "replay",
    data?: StateFileLink | ReplayFileLink,
}

export interface FrontendConfig {
    environment: Environment,
    work: Work,
    instance: Instance,
    manifest: ObjectFileLink[],
    saves: SaveFileLink[],
    start: PlayerStartTemplateInfo,
    boot_into_record:boolean,
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
    screenshot_id: string,
    replay_id?: string,
    creator_id?: string,
    state_replay_index?: number,
    state_derived_from?: string,
    save_derived_from?: string,
    created_on?: Date,
    file_id: string,
    file_hash: string,
    file_filename: string,
    file_source_path: string,
    file_dest_path: string
}

export interface SaveFileLink {
    save_id: string,
    instance_id: string,
    save_short_desc: string,
    save_description: string,
    creator_id?: string,
    save_derived_from?: string,
    state_derived_from?: string,
    replay_derived_from?: string,
    created_on?: Date,
    file_id: string,
    file_hash: string,
    file_filename: string,
    file_source_path: string,
    file_dest_path: string
}

export interface ReplayFileLink {
    replay_id: string,
    replay_name: string,
    replay_description: string,
    instance_id: string,
    creator_id: string,
    replay_forked_from?: string,
    created_on?: Date,
    file_id: string,
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
    state_derived_from: string|null,
    save_derived_from: string|null,
    replay_derived_from: string|null,
    [key:string]: string | boolean | Date | null
}
export function generateSaveFields():DBField[] {
    return [
        {field_name: "save_id" , value_type: "string", editable: false},
        {field_name: "instance_id" , value_type:"string", editable: false},
        {field_name: "file_id" , value_type:"string", editable: false},
        {field_name: "save_short_desc" , value_type:"string", editable: true},
        {field_name: "save_description" , value_type:"string", editable: true},
        {field_name: "creator_id" , value_type:"string", editable: false},
        {field_name: "created_on" , value_type:"Date", editable: false},
        {field_name: "state_derived_from" , value_type:"string", editable: false},
        {field_name: "save_derived_from" , value_type:"string", editable: false},
        {field_name: "replay_derived_from" , value_type:"string", editable: false}
    ]
}

export interface State {
    state_id: string,
    instance_id: string,
    is_checkpoint: boolean,
    file_id: string,
    state_name: string,
    state_description: string,
    screenshot_id: string,
    replay_id: string|null,
    creator_id: string,
    state_derived_from: string|null,
    save_derived_from: string|null,
    created_on: Date,
    [key:string]: string | boolean | Date | null
}
export function generateStateFields():DBField[] {
    return [
        {field_name: "state_id" , value_type: "string", editable: false},
        {field_name: "instance_id" , value_type:"string", editable: false},
        {field_name: "file_id" , value_type:"string", editable: false},
        {field_name: "is_checkpoint" , value_type:"boolean", editable: false},
        {field_name: "state_name" , value_type:"string", editable: true},
        {field_name: "state_description" , value_type:"string", editable: true},
        {field_name: "screenshot_id" , value_type:"string", editable: false},
        {field_name: "replay_id" , value_type:"string", editable: false},
        {field_name: "creator_id" , value_type:"string", editable: false},
        {field_name: "state_derived_from" , value_type:"string", editable: false},
        {field_name: "save_derived_from" , value_type:"string", editable: false},
        {field_name: "created_on" , value_type:"Date", editable: false}
    ]
}

export interface Replay {
    replay_id: string,
    replay_name: string,
    replay_description: string,
    instance_id: string,
    creator_id: string,
    replay_forked_from: string|null,
    file_id: string,
    created_on: Date,
    [key:string]: string | Date | null
}
export function generateReplayFields():DBField[] {
    return [
        {field_name: "replay_id" , value_type: "string", editable: false},
        {field_name: "replay_name" , value_type: "string", editable: true},
        {field_name: "replay_description" , value_type: "string", editable: true},
        {field_name: "instance_id" , value_type:"string", editable: false},
        {field_name: "creator_id" , value_type:"string", editable: false},
        {field_name: "replay_forked_from" , value_type:"string", editable: false},
        {field_name: "file_id" , value_type:"string", editable: false},
        {field_name: "created_on" , value_type:"Date", editable: false}
    ]
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

export interface ScreenshotCreateData {
    screenshot_data: string
}

export interface FullInstance {
    info: Instance,
    work: Work,
    states: State[],
    replays: Replay[],
    saves: Save[]
}

export interface InputLogEvent<T> {
  t: number,
  evt:T
}

export const NEVER_UPLOADED_ID = "DECAFBADDECAFBADDECAFBADDECAFBAD";
