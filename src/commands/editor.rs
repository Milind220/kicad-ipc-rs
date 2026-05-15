#[derive(Debug, Default)]
pub struct EditorCommands;

use crate::commands::base::pack_command;
use crate::proto::kiapi::common::commands as common_commands;

pub const GET_PAGE_SETTINGS: &str = "kiapi.common.commands.GetPageSettings";
pub const SET_PAGE_SETTINGS: &str = "kiapi.common.commands.SetPageSettings";
pub const PAGE_SETTINGS_RESPONSE: &str = "kiapi.common.types.PageSettings";

pub fn get_page_settings(command: common_commands::GetPageSettings) -> prost_types::Any {
    pack_command(&command, GET_PAGE_SETTINGS)
}

pub fn set_page_settings(command: common_commands::SetPageSettings) -> prost_types::Any {
    pack_command(&command, SET_PAGE_SETTINGS)
}
