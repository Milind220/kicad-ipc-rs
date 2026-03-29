//! Document operations: save, revert, title block, and string serialization.

use super::decode::decode_pcb_items;
use super::mappers::*;
use super::{
    model_document_to_proto, KiCadClient, CMD_GET_ITEMS_BY_ID, CMD_GET_TITLE_BLOCK_INFO,
    CMD_REVERT_DOCUMENT, CMD_SAVE_COPY_OF_DOCUMENT, CMD_SAVE_DOCUMENT, CMD_SAVE_DOCUMENT_TO_STRING,
    CMD_SAVE_SELECTION_TO_STRING, RES_GET_ITEMS_RESPONSE, RES_PROTOBUF_EMPTY,
    RES_SAVED_DOCUMENT_RESPONSE, RES_SAVED_SELECTION_RESPONSE, RES_TITLE_BLOCK_INFO,
};
use crate::envelope;
use crate::error::KiCadError;
use crate::model::board::*;
use crate::model::common::*;
use crate::proto::kiapi::common::commands as common_commands;
use crate::proto::kiapi::common::types as common_types;

impl KiCadClient {
    /// Reads title block metadata from the active PCB document.
    pub async fn get_title_block_info(&self) -> Result<TitleBlockInfo, KiCadError> {
        let command = common_commands::GetTitleBlockInfo {
            document: Some(self.current_board_document_proto().await?),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_TITLE_BLOCK_INFO))
            .await?;
        let payload: common_types::TitleBlockInfo =
            envelope::unpack_any(&response, RES_TITLE_BLOCK_INFO)?;

        let comments = vec![
            payload.comment1,
            payload.comment2,
            payload.comment3,
            payload.comment4,
            payload.comment5,
            payload.comment6,
            payload.comment7,
            payload.comment8,
            payload.comment9,
        ]
        .into_iter()
        .filter(|comment| !comment.is_empty())
        .collect();

        Ok(TitleBlockInfo {
            title: payload.title,
            date: payload.date,
            revision: payload.revision,
            company: payload.company,
            comments,
        })
    }

    pub async fn save_document_raw(&self) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::SaveDocument {
            document: Some(self.current_board_document_proto().await?),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_SAVE_DOCUMENT))
            .await?;
        response_payload_as_any(response, RES_PROTOBUF_EMPTY)
    }

    pub async fn save_document(&self) -> Result<(), KiCadError> {
        let _ = self.save_document_raw().await?;
        Ok(())
    }

    pub async fn save_copy_of_document_raw(
        &self,
        path: impl Into<String>,
        overwrite: bool,
        include_project: bool,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::SaveCopyOfDocument {
            document: Some(self.current_board_document_proto().await?),
            path: path.into(),
            options: Some(common_commands::SaveOptions {
                overwrite,
                include_project,
            }),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_SAVE_COPY_OF_DOCUMENT))
            .await?;
        response_payload_as_any(response, RES_PROTOBUF_EMPTY)
    }

    pub async fn save_copy_of_document(
        &self,
        path: impl Into<String>,
        overwrite: bool,
        include_project: bool,
    ) -> Result<(), KiCadError> {
        let _ = self
            .save_copy_of_document_raw(path, overwrite, include_project)
            .await?;
        Ok(())
    }

    pub async fn revert_document_raw(&self) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::RevertDocument {
            document: Some(self.current_board_document_proto().await?),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_REVERT_DOCUMENT))
            .await?;
        response_payload_as_any(response, RES_PROTOBUF_EMPTY)
    }

    pub async fn revert_document(&self) -> Result<(), KiCadError> {
        let _ = self.revert_document_raw().await?;
        Ok(())
    }

    /// Serializes the active PCB document to KiCad's string format.
    pub async fn get_board_as_string(&self) -> Result<String, KiCadError> {
        let command = common_commands::SaveDocumentToString {
            document: Some(self.current_board_document_proto().await?),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_SAVE_DOCUMENT_TO_STRING))
            .await?;
        let payload: common_commands::SavedDocumentResponse =
            envelope::unpack_any(&response, RES_SAVED_DOCUMENT_RESPONSE)?;
        Ok(payload.contents)
    }

    /// Serializes current selection to KiCad's string format.
    pub async fn get_selection_as_string(&self) -> Result<SelectionStringDump, KiCadError> {
        let command = common_commands::SaveSelectionToString {};

        let response = self
            .send_command(envelope::pack_any(&command, CMD_SAVE_SELECTION_TO_STRING))
            .await?;
        let payload: common_commands::SavedSelectionResponse =
            envelope::unpack_any(&response, RES_SAVED_SELECTION_RESPONSE)?;
        Ok(SelectionStringDump {
            ids: payload.ids.into_iter().map(|id| id.value).collect(),
            contents: payload.contents,
        })
    }

    pub async fn get_items_by_id_raw(
        &self,
        item_ids: Vec<String>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }

        let command = common_commands::GetItemsById {
            header: Some(self.current_board_item_header().await?),
            items: item_ids
                .into_iter()
                .map(|id| common_types::Kiid { value: id })
                .collect(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_ITEMS_BY_ID))
            .await?;

        let payload: common_commands::GetItemsResponse =
            envelope::unpack_any(&response, RES_GET_ITEMS_RESPONSE)?;

        ensure_item_request_ok(payload.status)?;
        Ok(payload.items)
    }

    pub async fn get_items_by_id_details(
        &self,
        item_ids: Vec<String>,
    ) -> Result<Vec<SelectionItemDetail>, KiCadError> {
        let items = self.get_items_by_id_raw(item_ids).await?;
        summarize_item_details(items)
    }

    /// Fetches and decodes items by KiCad item id.
    pub async fn get_items_by_id(&self, item_ids: Vec<String>) -> Result<Vec<PcbItem>, KiCadError> {
        let items = self.get_items_by_id_raw(item_ids).await?;
        decode_pcb_items(items)
    }
}
