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
  file_dest_path:string,
  file_filename:string,
  file_source_path:string,
  file_hash:string,
  object_id:string,
}

export interface ColdStart {
  type:string
}

export interface StartStateData {
  is_checkpoint:boolean,
  state_description:string,
  state_id:string,
  file_filename:string,
  file_hash:string,
  file_source_path:string,
  file_dest_path:string
}

export interface StateStart {
  type:string,
  data:StartStateData
}

export interface StartReplayData {
  file_filename:string,
  file_hash:string,
  replay_id:string,
  file_source_path:string,
  file_dest_path:string
}

export interface ReplayStart {
  type:string,
  data:StartReplayData
}


export interface SavefileInfo {
  file:string;
}
export interface StatefileInfo {
  file:string;
  thumbnail:string;
}
export interface ReplayfileInfo {
  file:string;
}
export interface ReplayCheckpointInfo {
  added:[StatefileInfo];
  delete_old?:boolean;
}
