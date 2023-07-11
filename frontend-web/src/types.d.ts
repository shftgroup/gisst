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
  object_dest_path:string,
  object_filename:string,
  object_source_path:string,
  object_hash:string,
  object_id:string,
}

export interface ColdStart {
  type:string
}

export interface StartStateData {
  is_checkpoint:boolean,
  state_description:string,
  state_filename:string,
  state_hash:string,
  state_id:string,
  state_path:string
}

export interface StateStart {
  type:string,
  data:StartStateData
}

export interface StartReplayData {
  replay_filename:string,
  replay_hash:string,
  replay_id:string,
  replay_path:string
}

export interface ReplayStart {
  type:string,
  data:StartReplayData
}
