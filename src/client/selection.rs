//! Selection management: get, add, remove, and clear the active PCB selection.

use crate::envelope;
use crate::error::KiCadError;
use crate::model::board::*;
use crate::model::common::*;
use crate::proto::kiapi::common::commands as common_commands;
use crate::proto::kiapi::common::types as common_types;

use super::decode::*;
use super::mappers::*;
use super::{
    KiCadClient, CMD_ADD_TO_SELECTION, CMD_CLEAR_SELECTION, CMD_GET_SELECTION,
    CMD_REMOVE_FROM_SELECTION, RES_PROTOBUF_EMPTY, RES_SELECTION_RESPONSE,
};

impl KiCadClient {
    /// Returns summarized counts for the current selection, optionally filtered by type codes.
    pub async fn get_selection_summary(
        &self,
        type_codes: Vec<i32>,
    ) -> Result<SelectionSummary, KiCadError> {
        let document = self.current_board_document_proto().await?;
        let command = common_commands::GetSelection {
            header: Some(common_types::ItemHeader {
                document: Some(document),
                container: None,
                field_mask: None,
            }),
            types: type_codes,
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_SELECTION))
            .await?;

        let payload: common_commands::SelectionResponse =
            envelope::unpack_any(&response, RES_SELECTION_RESPONSE)?;

        Ok(summarize_selection(&payload.items))
    }

    /// Returns current selection items as raw protobuf payloads.
    pub async fn get_selection_raw(
        &self,
        type_codes: Vec<i32>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        let command = common_commands::GetSelection {
            header: Some(self.current_board_item_header().await?),
            types: type_codes,
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_SELECTION))
            .await?;

        let payload: common_commands::SelectionResponse =
            envelope::unpack_any(&response, RES_SELECTION_RESPONSE)?;

        Ok(payload.items)
    }

    /// Returns lightweight detail rows for the current selection.
    pub async fn get_selection_details(
        &self,
        type_codes: Vec<i32>,
    ) -> Result<Vec<SelectionItemDetail>, KiCadError> {
        let items = self.get_selection_raw(type_codes).await?;
        summarize_item_details(items)
    }

    /// Returns the current selection as decoded typed PCB items.
    pub async fn get_selection(&self, type_codes: Vec<i32>) -> Result<Vec<PcbItem>, KiCadError> {
        let items = self.get_selection_raw(type_codes).await?;
        decode_pcb_items(items)
    }

    /// Adds item ids to the current selection and returns raw selection payloads.
    pub async fn add_to_selection_raw(
        &self,
        item_ids: Vec<String>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        let command = common_commands::AddToSelection {
            header: Some(self.current_board_item_header().await?),
            items: item_ids
                .into_iter()
                .map(|value| common_types::Kiid { value })
                .collect(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_ADD_TO_SELECTION))
            .await?;

        match envelope::unpack_any::<common_commands::SelectionResponse>(
            &response,
            RES_SELECTION_RESPONSE,
        ) {
            Ok(payload) => Ok(payload.items),
            Err(KiCadError::UnexpectedPayloadType {
                expected_type_url: _,
                actual_type_url,
            }) if actual_type_url == envelope::type_url(RES_PROTOBUF_EMPTY) => Ok(Vec::new()),
            Err(err) => Err(err),
        }
    }

    /// Adds item ids to the current selection and returns typed items with summary.
    pub async fn add_to_selection(
        &self,
        item_ids: Vec<String>,
    ) -> Result<SelectionMutationResult, KiCadError> {
        let raw_items = self.add_to_selection_raw(item_ids).await?;
        let summary = summarize_selection(&raw_items);
        let items = decode_pcb_items(raw_items)?;
        Ok(SelectionMutationResult { items, summary })
    }

    /// Clears the current selection and returns raw selection payloads.
    pub async fn clear_selection_raw(&self) -> Result<Vec<prost_types::Any>, KiCadError> {
        let command = common_commands::ClearSelection {
            header: Some(self.current_board_item_header().await?),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_CLEAR_SELECTION))
            .await?;

        match envelope::unpack_any::<common_commands::SelectionResponse>(
            &response,
            RES_SELECTION_RESPONSE,
        ) {
            Ok(payload) => Ok(payload.items),
            Err(KiCadError::UnexpectedPayloadType {
                expected_type_url: _,
                actual_type_url,
            }) if actual_type_url == envelope::type_url(RES_PROTOBUF_EMPTY) => Ok(Vec::new()),
            Err(err) => Err(err),
        }
    }

    /// Clears the current selection and returns typed items with summary.
    pub async fn clear_selection(&self) -> Result<SelectionMutationResult, KiCadError> {
        let raw_items = self.clear_selection_raw().await?;
        let summary = summarize_selection(&raw_items);
        let items = decode_pcb_items(raw_items)?;
        Ok(SelectionMutationResult { items, summary })
    }

    /// Removes item ids from the current selection and returns raw selection payloads.
    pub async fn remove_from_selection_raw(
        &self,
        item_ids: Vec<String>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        let command = common_commands::RemoveFromSelection {
            header: Some(self.current_board_item_header().await?),
            items: item_ids
                .into_iter()
                .map(|value| common_types::Kiid { value })
                .collect(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_REMOVE_FROM_SELECTION))
            .await?;

        match envelope::unpack_any::<common_commands::SelectionResponse>(
            &response,
            RES_SELECTION_RESPONSE,
        ) {
            Ok(payload) => Ok(payload.items),
            Err(KiCadError::UnexpectedPayloadType {
                expected_type_url: _,
                actual_type_url,
            }) if actual_type_url == envelope::type_url(RES_PROTOBUF_EMPTY) => Ok(Vec::new()),
            Err(err) => Err(err),
        }
    }

    /// Removes item ids from the current selection and returns typed items with summary.
    pub async fn remove_from_selection(
        &self,
        item_ids: Vec<String>,
    ) -> Result<SelectionMutationResult, KiCadError> {
        let raw_items = self.remove_from_selection_raw(item_ids).await?;
        let summary = summarize_selection(&raw_items);
        let items = decode_pcb_items(raw_items)?;
        Ok(SelectionMutationResult { items, summary })
    }
}
