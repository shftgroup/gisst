use crate::auth::LoggedInUserInfo;
use gisst::models::{
    CoreFileLink, Environment, Instance, ObjectLink, ReplayLink, SaveLink, StateLink, Work,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct PlayerTemplateInfo {
    pub gisst_root: String,
    pub instance: Instance,
    pub work: Work,
    pub environment: Environment,
    pub user: LoggedInUserInfo,
    pub saves: Vec<SaveLink>,
    pub start: PlayerStartTemplateInfo,
    pub core_manifest: Vec<CoreFileLink>,
    pub manifest: Vec<ObjectLink>,
    pub boot_into_record: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EmbedDataInfo {
    pub gisst_root: String,
    pub instance: Instance,
    pub work: Work,
    pub environment: Environment,
    pub saves: Vec<SaveLink>,
    pub start: PlayerStartTemplateInfo,
    pub core_manifest: Vec<CoreFileLink>,
    pub manifest: Vec<ObjectLink>,
    pub host_url: String,
    pub host_protocol: String,
    pub citation_data: Option<CitationDataInfo>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CitationDataInfo {
    pub website_title: String,
    pub url: String,
    pub gs_page_view_date: String,
    pub mla_page_view_date: String,
    pub bibtex_page_view_date: String,
    pub site_published_year: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum PlayerStartTemplateInfo {
    Cold,
    State(StateLink),
    Replay(ReplayLink),
}
