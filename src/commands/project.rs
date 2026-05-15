#[derive(Debug, Default)]
pub struct ProjectCommands;

use crate::commands::base::pack_command;
use crate::proto::kiapi::common::commands as common_commands;

pub const OPEN_DOCUMENT: &str = "kiapi.common.commands.OpenDocument";
pub const OPEN_DOCUMENT_RESPONSE: &str = "kiapi.common.commands.OpenDocumentResponse";
pub const CLOSE_DOCUMENT: &str = "kiapi.common.commands.CloseDocument";
pub const SAVE_DOCUMENT: &str = "kiapi.common.commands.SaveDocument";

pub fn open_document(command: common_commands::OpenDocument) -> prost_types::Any {
    pack_command(&command, OPEN_DOCUMENT)
}

pub fn close_document(command: common_commands::CloseDocument) -> prost_types::Any {
    pack_command(&command, CLOSE_DOCUMENT)
}

pub fn save_document(command: common_commands::SaveDocument) -> prost_types::Any {
    pack_command(&command, SAVE_DOCUMENT)
}
