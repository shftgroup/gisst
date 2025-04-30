export interface EmbedOptions {
  controls: ControllerOverlayMode
}

export enum ControllerOverlayMode {
  Off="off",
  On="on",
  Auto="auto"
}


export interface Environment {
  created_on:string,
  environment_config:object,
  environment_core_name:string,
  environment_core_version:string,
  environment_derived_from:string,
  environment_framework:string,
  environment_id:string,
  environment_name:string
}
export interface ObjectLink {
  object_role:string,
  object_role_index:integer,
  object_id:string,
  file_dest_path:string,
  file_filename:string,
  file_source_path:string,
  file_hash:string,
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

export interface ColdStart {
  type:string
}

export interface StartStateData {
  is_checkpoint:boolean,
  state_description:string,
  state_id:string,
  file_dest_path:string,
  file_filename:string,
  file_source_path:string,
  file_hash:string,
}

export interface StateStart {
  type:string,
  data:StartStateData
}

export interface StartReplayData {
  replay_id:string,
  file_dest_path:string,
  file_filename:string,
  file_source_path:string,
  file_hash:string,
}

export interface ReplayStart {
  type:string,
  data:StartReplayData
}


export interface EmuControls {
  toggle_mute();
  halt():Promise<void>;
}
