//! Item CRUD operations: create, update, delete, query, and commit workflows.

use super::decode::*;
use super::format::*;
use super::mappers::*;
use super::{
    KiCadClient, CMD_BEGIN_COMMIT, CMD_CREATE_ITEMS, CMD_DELETE_ITEMS, CMD_END_COMMIT,
    CMD_GET_CONNECTED_ITEMS, CMD_GET_ITEMS_BY_NET, CMD_GET_ITEMS_BY_NET_CLASS,
    CMD_GET_NETCLASS_FOR_NETS, CMD_PARSE_AND_CREATE_ITEMS_FROM_STRING, CMD_UPDATE_ITEMS,
    PCB_OBJECT_TYPES, RES_BEGIN_COMMIT_RESPONSE, RES_CREATE_ITEMS_RESPONSE,
    RES_DELETE_ITEMS_RESPONSE, RES_END_COMMIT_RESPONSE, RES_GET_ITEMS_RESPONSE,
    RES_NETCLASS_FOR_NETS_RESPONSE, RES_UPDATE_ITEMS_RESPONSE,
};
use crate::envelope;
use crate::error::KiCadError;
use crate::model::board::*;
use crate::model::common::*;
use crate::model::editable::*;
use crate::pcb_item_type_urls;
use crate::proto::kiapi::board::commands as board_commands;
use crate::proto::kiapi::board::types as board_types;
use crate::proto::kiapi::common::commands as common_commands;
use crate::proto::kiapi::common::types as common_types;

impl KiCadClient {
    /// Starts a commit session and returns the raw begin-commit payload.
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

    /// Ends a commit session and returns the raw end-commit payload.
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

    /// Creates items and returns the raw create-items payload.
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

    /// Creates editable items in the active PCB document.
    ///
    /// This is an ergonomic wrapper around [`KiCadClient::create_items`] that
    /// keeps item payloads in editable form.
    pub async fn create_editable_items(
        &self,
        items: Vec<EditablePcbItem>,
        container_id: Option<String>,
    ) -> Result<Vec<EditablePcbItem>, KiCadError> {
        let items: Vec<prost_types::Any> = items.into_iter().map(Into::into).collect();
        let created_items = self.create_items(items, container_id).await?;
        created_items
            .into_iter()
            .map(EditablePcbItem::try_from)
            .collect()
    }

    /// Creates one board text item through the same typed `CreateItems` path
    /// used by official `kicad-python` `BoardText` objects.
    pub async fn create_board_text(&self, spec: BoardTextSpec) -> Result<PcbBoardText, KiCadError> {
        let mut created = self.create_board_texts(vec![spec]).await?;
        created.pop().ok_or_else(|| KiCadError::InvalidResponse {
            reason: "CreateItems returned no board text item".to_string(),
        })
    }

    /// Creates one board text item inside an existing board container.
    pub async fn create_board_text_in_container(
        &self,
        spec: BoardTextSpec,
        container_id: String,
    ) -> Result<PcbBoardText, KiCadError> {
        let mut created = self
            .create_board_texts_in_container(vec![spec], container_id)
            .await?;
        created.pop().ok_or_else(|| KiCadError::InvalidResponse {
            reason: "CreateItems returned no board text item".to_string(),
        })
    }

    /// Creates board text items through typed `CreateItems`.
    pub async fn create_board_texts(
        &self,
        specs: Vec<BoardTextSpec>,
    ) -> Result<Vec<PcbBoardText>, KiCadError> {
        self.create_board_texts_with_container(specs, None).await
    }

    /// Creates board text items inside an existing board container.
    pub async fn create_board_texts_in_container(
        &self,
        specs: Vec<BoardTextSpec>,
        container_id: String,
    ) -> Result<Vec<PcbBoardText>, KiCadError> {
        self.create_board_texts_with_container(specs, Some(container_id))
            .await
    }

    async fn create_board_texts_with_container(
        &self,
        specs: Vec<BoardTextSpec>,
        container_id: Option<String>,
    ) -> Result<Vec<PcbBoardText>, KiCadError> {
        let items = specs.into_iter().map(board_text_spec_to_any).collect();
        let created = self.create_items(items, container_id).await?;
        created
            .into_iter()
            .map(|item| match decode_pcb_item(item)? {
                PcbItem::BoardText(text) => Ok(text),
                other => Err(KiCadError::InvalidResponse {
                    reason: format!("CreateItems returned non-board-text item: {other:?}"),
                }),
            })
            .collect()
    }

    /// Updates items and returns the raw update-items payload.
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

    /// Updates editable items in the active PCB document.
    ///
    /// This is an ergonomic wrapper around [`KiCadClient::update_items`] that
    /// keeps item payloads in editable form.
    pub async fn update_editable_items(
        &self,
        items: Vec<EditablePcbItem>,
    ) -> Result<Vec<EditablePcbItem>, KiCadError> {
        let items: Vec<prost_types::Any> = items.into_iter().map(Into::into).collect();
        let updated_items = self.update_items(items).await?;
        updated_items
            .into_iter()
            .map(EditablePcbItem::try_from)
            .collect()
    }

    /// Deletes items and returns the raw delete-items payload.
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
    /// Returns ids that KiCad reported as deleted.
    ///
    /// KiCad 10.0.x can acknowledge `DeleteItems` but omit per-item result rows;
    /// in that case this method returns the requested ids after request success.
    /// Treat those ids as accepted by KiCad, not independently verified deleted.
    pub async fn delete_items(&self, item_ids: Vec<String>) -> Result<Vec<String>, KiCadError> {
        let requested_item_ids = item_ids.clone();
        let payload = self.delete_items_raw(item_ids).await?;
        let response: common_commands::DeleteItemsResponse =
            decode_any(&payload, RES_DELETE_ITEMS_RESPONSE)?;
        deleted_item_ids_from_response(requested_item_ids, response)
    }

    /// Parses KiCad item text and creates items, returning raw create-items payload.
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

    /// Parses KiCad item text and returns created items as raw payloads.
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

    /// Returns `(pad_id, net)` mappings derived from footprint items.
    pub async fn get_pad_netlist(&self) -> Result<Vec<PadNetEntry>, KiCadError> {
        let footprint_items = self
            .get_items_raw(vec![common_types::KiCadObjectType::KotPcbFootprint as i32])
            .await?;
        pad_netlist_from_footprint_items(footprint_items)
    }
    /// Returns vias as raw protobuf payloads.
    pub async fn get_vias_raw(&self) -> Result<Vec<prost_types::Any>, KiCadError> {
        self.get_items_raw(vec![common_types::KiCadObjectType::KotPcbVia as i32])
            .await
    }

    /// Returns vias decoded into typed [`PcbVia`] entries.
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

    /// Fetches items by object type codes and returns raw protobuf payloads.
    pub async fn get_items_raw_by_type_codes(
        &self,
        type_codes: Vec<i32>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        self.get_items_raw(type_codes).await
    }

    /// Fetches item details by object type codes.
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

    /// Fetches editable items by KiCad object type codes.
    pub async fn get_editable_items_by_type_codes(
        &self,
        type_codes: Vec<i32>,
    ) -> Result<Vec<EditablePcbItem>, KiCadError> {
        let items = self.get_items_raw(type_codes).await?;
        items.into_iter().map(EditablePcbItem::try_from).collect()
    }

    /// Fetches all known object type buckets and returns raw payloads.
    pub async fn get_all_pcb_items_raw(
        &self,
    ) -> Result<Vec<(PcbObjectTypeCode, Vec<prost_types::Any>)>, KiCadError> {
        let items = self
            .get_items_raw(
                PCB_OBJECT_TYPES
                    .iter()
                    .map(|object_type| object_type.code)
                    .collect(),
            )
            .await?;
        bucket_items_by_pcb_object_type(items)
    }

    /// Fetches all known object type buckets and returns decoded detail rows.
    pub async fn get_all_pcb_items_details(
        &self,
    ) -> Result<Vec<(PcbObjectTypeCode, Vec<SelectionItemDetail>)>, KiCadError> {
        self.get_all_pcb_items_raw()
            .await?
            .into_iter()
            .map(|(object_type, items)| Ok((object_type, summarize_item_details(items)?)))
            .collect()
    }

    /// Fetches all known PCB item kinds and decodes each bucket.
    pub async fn get_all_pcb_items(
        &self,
    ) -> Result<Vec<(PcbObjectTypeCode, Vec<PcbItem>)>, KiCadError> {
        self.get_all_pcb_items_raw()
            .await?
            .into_iter()
            .map(|(object_type, items)| Ok((object_type, decode_pcb_items(items)?)))
            .collect()
    }

    /// Fetches items filtered by nets and returns raw protobuf payloads.
    ///
    /// KiCad 10.0.1 treats net names as authoritative for this command. Net
    /// codes are still sent for compatibility with legacy payloads, but names
    /// should be considered the canonical identifiers.
    pub async fn get_items_by_net_raw(
        &self,
        type_codes: Vec<i32>,
        nets: Vec<BoardNet>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        let command = build_get_items_by_net_command(
            self.current_board_item_header().await?,
            type_codes,
            nets,
        );

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_ITEMS_BY_NET))
            .await?;
        let payload: common_commands::GetItemsResponse =
            envelope::unpack_any(&response, RES_GET_ITEMS_RESPONSE)?;
        ensure_item_request_ok(payload.status)?;
        Ok(payload.items)
    }

    /// Fetches items filtered by nets and decodes typed items.
    pub async fn get_items_by_net(
        &self,
        type_codes: Vec<i32>,
        nets: Vec<BoardNet>,
    ) -> Result<Vec<PcbItem>, KiCadError> {
        let items = self.get_items_by_net_raw(type_codes, nets).await?;
        decode_pcb_items(items)
    }
    /// Fetches items filtered by net class names and returns raw payloads.
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

    /// Fetches items filtered by net class names and decodes typed items.
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

    /// Fetches copper-connected items from one or more source item ids and returns raw payloads.
    pub async fn get_connected_items_raw(
        &self,
        item_ids: Vec<String>,
        type_codes: Vec<i32>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }

        let command = build_get_connected_items_command(
            self.current_board_item_header().await?,
            item_ids,
            type_codes,
        );
        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_CONNECTED_ITEMS))
            .await?;
        let payload: common_commands::GetItemsResponse =
            envelope::unpack_any(&response, RES_GET_ITEMS_RESPONSE)?;
        ensure_item_request_ok(payload.status)?;
        Ok(payload.items)
    }

    /// Fetches copper-connected items from one or more source item ids.
    pub async fn get_connected_items(
        &self,
        item_ids: Vec<String>,
        type_codes: Vec<i32>,
    ) -> Result<Vec<PcbItem>, KiCadError> {
        let items = self.get_connected_items_raw(item_ids, type_codes).await?;
        decode_pcb_items(items)
    }

    /// Resolves net class assignments for nets and returns raw response payload.
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

    /// Resolves net class assignments for nets.
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

pub(crate) fn pcb_object_type_for_any(item: &prost_types::Any) -> Option<PcbObjectTypeCode> {
    let type_name = item
        .type_url
        .strip_prefix("type.googleapis.com/")
        .unwrap_or(item.type_url.as_str());

    let code = match type_name {
        pcb_item_type_urls::FOOTPRINT_INSTANCE => {
            common_types::KiCadObjectType::KotPcbFootprint as i32
        }
        pcb_item_type_urls::PAD => common_types::KiCadObjectType::KotPcbPad as i32,
        pcb_item_type_urls::BOARD_GRAPHIC_SHAPE => {
            common_types::KiCadObjectType::KotPcbShape as i32
        }
        pcb_item_type_urls::REFERENCE_IMAGE => {
            common_types::KiCadObjectType::KotPcbReferenceImage as i32
        }
        pcb_item_type_urls::FIELD => common_types::KiCadObjectType::KotPcbField as i32,
        pcb_item_type_urls::BOARD_TEXT => common_types::KiCadObjectType::KotPcbText as i32,
        pcb_item_type_urls::BOARD_TEXT_BOX => common_types::KiCadObjectType::KotPcbTextbox as i32,
        pcb_item_type_urls::TRACK => common_types::KiCadObjectType::KotPcbTrace as i32,
        pcb_item_type_urls::VIA => common_types::KiCadObjectType::KotPcbVia as i32,
        pcb_item_type_urls::ARC => common_types::KiCadObjectType::KotPcbArc as i32,
        pcb_item_type_urls::DIMENSION => common_types::KiCadObjectType::KotPcbDimension as i32,
        pcb_item_type_urls::ZONE => common_types::KiCadObjectType::KotPcbZone as i32,
        pcb_item_type_urls::GROUP => common_types::KiCadObjectType::KotPcbGroup as i32,
        pcb_item_type_urls::BARCODE => common_types::KiCadObjectType::KotPcbBarcode as i32,
        _ => return None,
    };

    PcbObjectTypeCode::from_code(code)
}

pub(crate) fn bucket_items_by_pcb_object_type(
    items: Vec<prost_types::Any>,
) -> Result<Vec<(PcbObjectTypeCode, Vec<prost_types::Any>)>, KiCadError> {
    let mut rows: Vec<_> = PCB_OBJECT_TYPES
        .iter()
        .copied()
        .map(|object_type| (object_type, Vec::new()))
        .collect();

    for item in items {
        let type_url = item.type_url.clone();
        let object_type =
            pcb_object_type_for_any(&item).ok_or_else(|| KiCadError::InvalidResponse {
                reason: format!("GetItems returned unmapped PCB item type `{type_url}`"),
            })?;

        if let Some((_, bucket)) = rows
            .iter_mut()
            .find(|(row_type, _)| row_type.code == object_type.code)
        {
            bucket.push(item);
        }
    }

    Ok(rows)
}

pub(crate) fn deleted_item_ids_from_response(
    requested_item_ids: Vec<String>,
    response: common_commands::DeleteItemsResponse,
) -> Result<Vec<String>, KiCadError> {
    ensure_item_request_ok(response.status)?;

    if response.deleted_items.is_empty() {
        return Ok(requested_item_ids);
    }

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

fn board_nets_to_proto(nets: Vec<BoardNet>) -> Vec<board_types::Net> {
    nets.into_iter()
        .map(|net| board_types::Net {
            code: Some(board_types::NetCode { value: net.code }),
            name: net.name,
        })
        .collect()
}

fn build_get_items_by_net_command(
    header: common_types::ItemHeader,
    type_codes: Vec<i32>,
    nets: Vec<BoardNet>,
) -> board_commands::GetItemsByNet {
    board_commands::GetItemsByNet {
        header: Some(header),
        types: type_codes,
        nets: board_nets_to_proto(nets),
    }
}

fn build_get_connected_items_command(
    header: common_types::ItemHeader,
    item_ids: Vec<String>,
    type_codes: Vec<i32>,
) -> board_commands::GetConnectedItems {
    board_commands::GetConnectedItems {
        header: Some(header),
        items: item_ids
            .into_iter()
            .map(|value| common_types::Kiid { value })
            .collect(),
        types: type_codes,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_get_connected_items_command, build_get_items_by_net_command};
    use crate::model::board::BoardNet;
    use crate::proto::kiapi::common::types as common_types;

    fn sample_header() -> common_types::ItemHeader {
        common_types::ItemHeader {
            document: Some(common_types::DocumentSpecifier::default()),
            container: None,
            field_mask: None,
        }
    }

    #[test]
    fn get_items_by_net_command_maps_names_and_codes() {
        let command = build_get_items_by_net_command(
            sample_header(),
            vec![11, 12],
            vec![
                BoardNet {
                    code: 41,
                    name: "Net-(U1-Pad1)".to_string(),
                },
                BoardNet {
                    code: 0,
                    name: "GND".to_string(),
                },
            ],
        );

        assert_eq!(command.types, vec![11, 12]);
        assert_eq!(command.nets.len(), 2);
        assert_eq!(command.nets[0].name, "Net-(U1-Pad1)");
        assert_eq!(
            command.nets[0].code.as_ref().map(|code| code.value),
            Some(41)
        );
        assert_eq!(command.nets[1].name, "GND");
        assert_eq!(
            command.nets[1].code.as_ref().map(|code| code.value),
            Some(0)
        );
    }

    #[test]
    fn get_connected_items_command_maps_item_ids_and_types() {
        let command = build_get_connected_items_command(
            sample_header(),
            vec!["uuid-1".to_string(), "uuid-2".to_string()],
            vec![11, 12],
        );

        assert_eq!(command.types, vec![11, 12]);
        assert_eq!(command.items.len(), 2);
        assert_eq!(command.items[0].value, "uuid-1");
        assert_eq!(command.items[1].value, "uuid-2");
        assert!(command.header.is_some());
    }
}
