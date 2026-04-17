import {GISSTModels} from 'gisst-player';

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
  environment_platform:string
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

export interface CoreFileLink {
  core_name:string,
  core_version:string,
  core_role:string,
  core_role_index:integer,
  file_hash:string,
  file_filename:string,
  file_source_path:string,
  file_dest_path:string,
}

export interface ColdStart {
  type:string
}

export interface StateStart {
  type:string,
  data:GISSTModels.StateFileLink
}

export interface ReplayStart {
  type:string,
  data:GISSTModels.ReplayFileLink
}
