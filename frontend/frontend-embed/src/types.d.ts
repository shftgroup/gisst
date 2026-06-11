export interface EmbedOptions {
  controls: ControllerOverlayMode,
  record_from_start: bool
}

export enum ControllerOverlayMode {
  Off="off",
  On="on",
  Auto="auto"
}

export interface StringIndexable {
  [s:string] : StringIndexable | string;
}

export interface Environment {
  /* gisst::models::Environment */
  created_on:string,
  environment_config:StringIndexable,
  environment_core_name:string,
  environment_core_version:string,
  environment_derived_from:string,
  environment_framework:string,
  environment_id:string,
  environment_name:string
  environment_platform:string
}

export interface Work {
  /* gisst::models::Work */
  work_id: string,
  work_name: string,
  work_version: string,
  work_platform: string,
  created_on: string
}

export interface Instance {
  /* gisst::models::Instance */
  instance_id: string,
  environment_id: string,
  work_id: string,
  instance_config: JSON,
  created_on: string
}


export interface CoreFileLink {
  /* gisst::models::CoreFileLink */
  core_name:string,
  core_version:string,
  core_role:string,
  core_role_index:integer,
  file_hash:string,
  file_filename:string,
  file_source_path:string,
  file_dest_path:string,
}

export interface ObjectLink {
  /* gisst::models::ObjectLink */
  object_role:string,
  object_role_index:integer,
  object_id:string,
  file_dest_path:string,
  file_filename:string,
  file_source_path:string,
  file_hash:string,
}

export interface SaveFileLink {
  /* gisst::models::SaveLink */
  save_id: string,
  instance_id: string,
  save_short_desc: string,
  save_description: string,
  creator_id?: string,
  save_derived_from?: string,
  state_derived_from?: string,
  replay_derived_from?: string,
  created_on?: string,
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
  /* gisst::models::StateLink */
  is_checkpoint:boolean,
  state_description:string,
  state_id:string,
  file_dest_path:string,
  file_filename:string,
  file_source_path:string,
  file_hash:string,
  screenshot_id:string,
  created_on:string,
  state?:Uint8Array
}

export interface StateStart {
  type:string,
  data:StartStateData
}

export interface StartReplayData {
  /* gisst::models::ReplayLink */
  replay_id:string,
  file_dest_path:string,
  file_filename:string,
  file_source_path:string,
  file_hash:string,
  created_on:string,
  replay?:Uint8Array
}

export interface ReplayStart {
  type:string,
  data:StartReplayData
}

export interface EmuControls {
  toggle_mute();
  halt():Promise<void>;
  gisst_root:string,
  environment:Environment,
  work: Work,
  instance: Instance,
  start:ColdStart | StateStart | ReplayStart,
  saves:SaveFileLink[],
  core_manifest:CoreFileLink[],
  manifest:ObjectLink[],
  container:HTMLDivElement,
  embed_options:EmbedOptions,
  inner:RetroarchCore | EmbedV86,
  on_ready: ()=>void
}

export interface RetroarchCore {
  send_message(string):Promise<void>;
  read_response(bool):Promise<string>;
  module:LibretroModule,
  content_name:string,
  state_dir:string,
  saves_dir:string
}
