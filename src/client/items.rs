//! Item CRUD operations: create, update, delete, query, and commit workflows.

use std::collections::BTreeMap;

use crate::envelope;
use crate::error::KiCadError;
use crate::model::board::*;
use crate::model::common::*;
use crate::proto::kiapi::board::commands as board_commands;
use crate::proto::kiapi::board::types as board_types;
use crate::proto::kiapi::common::commands as common_commands;
use crate::proto::kiapi::common::types as common_types;

use super::decode::*;
use super::format::*;
use super::mappers::*;
use super::{
    model_document_to_proto, KiCadClient, CMD_BEGIN_COMMIT, CMD_CREATE_ITEMS, CMD_DELETE_ITEMS,
    CMD_END_COMMIT, CMD_GET_ITEMS, CMD_GET_ITEMS_BY_ID, CMD_GET_ITEMS_BY_NET,
    CMD_GET_ITEMS_BY_NET_CLASS, CMD_GET_NETCLASS_FOR_NETS, CMD_PARSE_AND_CREATE_ITEMS_FROM_STRING,
    CMD_UPDATE_ITEMS, PCB_OBJECT_TYPES, RES_BEGIN_COMMIT_RESPONSE, RES_CREATE_ITEMS_RESPONSE,
    RES_DELETE_ITEMS_RESPONSE, RES_END_COMMIT_RESPONSE, RES_GET_ITEMS_RESPONSE,
    RES_NETCLASS_FOR_NETS_RESPONSE, RES_PROTOBUF_EMPTY, RES_UPDATE_ITEMS_RESPONSE,
};

impl KiCadClient {
    pub async fn begin_commit_raw(&self) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::BeginCommit {};
        let response = self
            .send_command(envelope::pack_any(&command, CMD_BEGIN_COMMIT))
            .await?;
        response_payload_as_any(response, RES_BEGIN_COMMIT_RESPONSE)
    }

    /// Starts a KiCad commit session used for grouped board edits.
    pub async fn begin_commit(&self) -> Result<CommitSession, KiCadError> {
        let payload = self.begin_commit_raw().await?;
        let response: common_commands::BeginCommitResponse =
            decode_any(&payload, RES_BEGIN_COMMIT_RESPONSE)?;
        map_commit_session(response)
    }

    pub async fn end_commit_raw(
        &self,
        session: CommitSession,
        action: CommitAction,
        message: impl Into<String>,
    ) -> Result<prost_types::Any, KiCadError> {
        if session.id.is_empty() {
            return Err(KiCadError::Config {
                reason: "end_commit_raw requires a non-empty commit session id".to_string(),
            });
        }

        let command = common_commands::EndCommit {
            id: Some(common_types::Kiid { value: session.id }),
            action: commit_action_to_proto(action),
            message: message.into(),
        };
        let response = self
            .send_command(envelope::pack_any(&command, CMD_END_COMMIT))
            .await?;
        response_payload_as_any(response, RES_END_COMMIT_RESPONSE)
    }

    /// Finalizes a commit session, either committing or dropping staged changes.
    pub async fn end_commit(
        &self,
        session: CommitSession,
        action: CommitAction,
        message: impl Into<String>,
    ) -> Result<(), KiCadError> {
        self.end_commit_raw(session, action, message).await?;
        Ok(())
    }

    pub async fn create_items_raw(
        &self,
        items: Vec<prost_types::Any>,
        container_id: Option<String>,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::CreateItems {
            header: Some(self.current_board_item_header().await?),
            items,
            container: container_id.map(|value| common_types::Kiid { value }),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_CREATE_ITEMS))
            .await?;
        response_payload_as_any(response, RES_CREATE_ITEMS_RESPONSE)
    }

    /// Creates items in the active PCB document.
    ///
    /// Returns created items as raw protobuf `Any` payloads.
    pub async fn create_items(
        &self,
        items: Vec<prost_types::Any>,
        container_id: Option<String>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        let payload = self.create_items_raw(items, container_id).await?;
        let response: common_commands::CreateItemsResponse =
            decode_any(&payload, RES_CREATE_ITEMS_RESPONSE)?;
        ensure_item_request_ok(response.status)?;

        response
            .created_items
            .into_iter()
            .map(|row| {
                ensure_item_status_ok(row.status)?;
                row.item.ok_or_else(|| KiCadError::InvalidResponse {
                    reason: "CreateItemsResponse missing created item payload".to_string(),
                })
            })
            .collect()
    }

    pub async fn update_items_raw(
        &self,
        items: Vec<prost_types::Any>,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::UpdateItems {
            header: Some(self.current_board_item_header().await?),
            items,
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_UPDATE_ITEMS))
            .await?;
        response_payload_as_any(response, RES_UPDATE_ITEMS_RESPONSE)
    }

    /// Updates existing items in the active PCB document.
    ///
    /// Returns updated items as raw protobuf `Any` payloads.
    pub async fn update_items(
        &self,
        items: Vec<prost_types::Any>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        let payload = self.update_items_raw(items).await?;
        let response: common_commands::UpdateItemsResponse =
            decode_any(&payload, RES_UPDATE_ITEMS_RESPONSE)?;
        ensure_item_request_ok(response.status)?;

        response
            .updated_items
            .into_iter()
            .map(|row| {
                ensure_item_status_ok(row.status)?;
                row.item.ok_or_else(|| KiCadError::InvalidResponse {
                    reason: "UpdateItemsResponse missing updated item payload".to_string(),
                })
            })
            .collect()
    }

    pub async fn delete_items_raw(
        &self,
        item_ids: Vec<String>,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::DeleteItems {
            header: Some(self.current_board_item_header().await?),
            item_ids: item_ids
                .into_iter()
                .map(|value| common_types::Kiid { value })
                .collect(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_DELETE_ITEMS))
            .await?;
        response_payload_as_any(response, RES_DELETE_ITEMS_RESPONSE)
    }

    /// Deletes items by id from the active PCB document.
    ///
    /// Returns ids of items deleted by KiCad.
    pub async fn delete_items(&self, item_ids: Vec<String>) -> Result<Vec<String>, KiCadError> {
        let payload = self.delete_items_raw(item_ids).await?;
        let response: common_commands::DeleteItemsResponse =
            decode_any(&payload, RES_DELETE_ITEMS_RESPONSE)?;
        ensure_item_request_ok(response.status)?;

        response
            .deleted_items
            .into_iter()
            .map(|row| {
                ensure_item_deletion_status_ok(row.status)?;
                row.id
                    .map(|id| id.value)
                    .ok_or_else(|| KiCadError::InvalidResponse {
                        reason: "DeleteItemsResponse missing deleted item id".to_string(),
                    })
            })
            .collect()
    }

    pub async fn parse_and_create_items_from_string_raw(
        &self,
        contents: impl Into<String>,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::ParseAndCreateItemsFromString {
            document: Some(self.current_board_document_proto().await?),
            contents: contents.into(),
        };

        let response = self
            .send_command(envelope::pack_any(
                &command,
                CMD_PARSE_AND_CREATE_ITEMS_FROM_STRING,
            ))
            .await?;
        response_payload_as_any(response, RES_CREATE_ITEMS_RESPONSE)
    }

    pub async fn parse_and_create_items_from_string(
        &self,
        contents: impl Into<String>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        let payload = self
            .parse_and_create_items_from_string_raw(contents)
            .await?;
        let response: common_commands::CreateItemsResponse =
            decode_any(&payload, RES_CREATE_ITEMS_RESPONSE)?;
        ensure_item_request_ok(response.status)?;

        response
            .created_items
            .into_iter()
            .map(|row| {
                ensure_item_status_ok(row.status)?;
                row.item.ok_or_else(|| KiCadError::InvalidResponse {
                    reason: "CreateItemsResponse missing created item payload".to_string(),
                })
            })
            .collect()
    }

    pub async fn get_pad_netlist(&self) -> Result<Vec<PadNetEntry>, KiCadError> {
        let footprint_items = self
            .get_items_raw(vec![common_types::KiCadObjectType::KotPcbFootprint as i32])
            .await?;
        pad_netlist_from_footprint_items(footprint_items)
    }
    pub async fn get_vias_raw(&self) -> Result<Vec<prost_types::Any>, KiCadError> {
        self.get_items_raw(vec![common_types::KiCadObjectType::KotPcbVia as i32])
            .await
    }

    pub async fn get_vias(&self) -> Result<Vec<PcbVia>, KiCadError> {
        let items = self
            .get_items_by_type_codes(vec![common_types::KiCadObjectType::KotPcbVia as i32])
            .await?;
        Ok(items
            .into_iter()
            .filter_map(|item| match item {
                PcbItem::Via(via) => Some(via),
                _ => None,
            })
            .collect())
    }

    /// Returns known KiCad PCB object type codes handled by this crate.
    pub fn pcb_object_type_codes() -> &'static [PcbObjectTypeCode] {
        &PCB_OBJECT_TYPES
    }

    /// Resolves a human-readable object type name from a KiCad object type code.
    pub fn pcb_object_type_name(type_code: i32) -> Option<&'static str> {
        PCB_OBJECT_TYPES
            .iter()
            .find(|entry| entry.code == type_code)
            .map(|entry| entry.name)
    }

    /// Formats a raw protobuf PCB item payload for debugging/logging.
    pub fn debug_any_item(item: &prost_types::Any) -> Result<String, KiCadError> {
        any_to_pretty_debug(item)
    }

    pub async fn get_items_raw_by_type_codes(
        &self,
        type_codes: Vec<i32>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        self.get_items_raw(type_codes).await
    }

    pub async fn get_items_details_by_type_codes(
        &self,
        type_codes: Vec<i32>,
    ) -> Result<Vec<SelectionItemDetail>, KiCadError> {
        let items = self.get_items_raw(type_codes).await?;
        summarize_item_details(items)
    }

    /// Fetches and decodes items by KiCad object type codes.
    pub async fn get_items_by_type_codes(
        &self,
        type_codes: Vec<i32>,
    ) -> Result<Vec<PcbItem>, KiCadError> {
        let items = self.get_items_raw(type_codes).await?;
        decode_pcb_items(items)
    }

    pub async fn get_all_pcb_items_raw(
        &self,
    ) -> Result<Vec<(PcbObjectTypeCode, Vec<prost_types::Any>)>, KiCadError> {
        let mut rows = Vec::with_capacity(PCB_OBJECT_TYPES.len());
        for object_type in PCB_OBJECT_TYPES {
            let items = self.get_items_raw(vec![object_type.code]).await?;
            rows.push((object_type, items));
        }

        Ok(rows)
    }

    pub async fn get_all_pcb_items_details(
        &self,
    ) -> Result<Vec<(PcbObjectTypeCode, Vec<SelectionItemDetail>)>, KiCadError> {
        let mut rows = Vec::with_capacity(PCB_OBJECT_TYPES.len());
        for object_type in PCB_OBJECT_TYPES {
            let items = self.get_items_raw(vec![object_type.code]).await?;
            rows.push((object_type, summarize_item_details(items)?));
        }

        Ok(rows)
    }

    /// Fetches all known PCB item kinds and decodes each bucket.
    pub async fn get_all_pcb_items(
        &self,
    ) -> Result<Vec<(PcbObjectTypeCode, Vec<PcbItem>)>, KiCadError> {
        let mut rows = Vec::with_capacity(PCB_OBJECT_TYPES.len());
        for object_type in PCB_OBJECT_TYPES {
            let items = self.get_items_raw(vec![object_type.code]).await?;
            rows.push((object_type, decode_pcb_items(items)?));
        }

        Ok(rows)
    }

    pub async fn get_items_by_net_raw(
        &self,
        type_codes: Vec<i32>,
        net_codes: Vec<i32>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        let command = board_commands::GetItemsByNet {
            header: Some(self.current_board_item_header().await?),
            types: type_codes,
            net_codes: net_codes
                .into_iter()
                .map(|value| board_types::NetCode { value })
                .collect(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_ITEMS_BY_NET))
            .await?;
        let payload: common_commands::GetItemsResponse =
            envelope::unpack_any(&response, RES_GET_ITEMS_RESPONSE)?;
        ensure_item_request_ok(payload.status)?;
        Ok(payload.items)
    }

    pub async fn get_items_by_net(
        &self,
        type_codes: Vec<i32>,
        net_codes: Vec<i32>,
    ) -> Result<Vec<PcbItem>, KiCadError> {
        let items = self.get_items_by_net_raw(type_codes, net_codes).await?;
        decode_pcb_items(items)
    }

    pub async fn get_items_by_net_class_raw(
        &self,
        type_codes: Vec<i32>,
        net_classes: Vec<String>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        let command = board_commands::GetItemsByNetClass {
            header: Some(self.current_board_item_header().await?),
            types: type_codes,
            net_classes,
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_ITEMS_BY_NET_CLASS))
            .await?;
        let payload: common_commands::GetItemsResponse =
            envelope::unpack_any(&response, RES_GET_ITEMS_RESPONSE)?;
        ensure_item_request_ok(payload.status)?;
        Ok(payload.items)
    }

    pub async fn get_items_by_net_class(
        &self,
        type_codes: Vec<i32>,
        net_classes: Vec<String>,
    ) -> Result<Vec<PcbItem>, KiCadError> {
        let items = self
            .get_items_by_net_class_raw(type_codes, net_classes)
            .await?;
        decode_pcb_items(items)
    }

    pub async fn get_netclass_for_nets_raw(
        &self,
        nets: Vec<BoardNet>,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = board_commands::GetNetClassForNets {
            net: nets
                .into_iter()
                .map(|net| board_types::Net {
                    code: Some(board_types::NetCode { value: net.code }),
                    name: net.name,
                })
                .collect(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_NETCLASS_FOR_NETS))
            .await?;

        response_payload_as_any(response, RES_NETCLASS_FOR_NETS_RESPONSE)
    }

    pub async fn get_netclass_for_nets(
        &self,
        nets: Vec<BoardNet>,
    ) -> Result<Vec<NetClassForNetEntry>, KiCadError> {
        let payload = self.get_netclass_for_nets_raw(nets).await?;
        let response: board_commands::NetClassForNetsResponse =
            decode_any(&payload, RES_NETCLASS_FOR_NETS_RESPONSE)?;
        Ok(map_netclass_for_nets_response(response))
    }
}
