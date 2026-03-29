//! Geometry queries: bounding boxes, hit testing, pad polygons, padstack presence, and zone refill.

use crate::envelope;
use crate::error::KiCadError;
use crate::model::board::*;
use crate::model::common::*;
use crate::proto::kiapi::board::commands as board_commands;
use crate::proto::kiapi::common::commands as common_commands;
use crate::proto::kiapi::common::types as common_types;

use super::mappers::*;
use super::{
    KiCadClient, CMD_CHECK_PADSTACK_PRESENCE_ON_LAYERS, CMD_GET_BOUNDING_BOX,
    CMD_GET_PAD_SHAPE_AS_POLYGON, CMD_HIT_TEST, CMD_REFILL_ZONES, PAD_QUERY_CHUNK_SIZE,
    RES_GET_BOUNDING_BOX_RESPONSE, RES_HIT_TEST_RESPONSE, RES_PADSTACK_PRESENCE_RESPONSE,
    RES_PAD_SHAPE_AS_POLYGON_RESPONSE, RES_PROTOBUF_EMPTY,
};

impl KiCadClient {
    /// Rebuilds fill geometry for the given zone ids.
    pub async fn refill_zones(&self, zone_ids: Vec<String>) -> Result<(), KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::RefillZones {
            board: Some(board),
            zones: zone_ids
                .into_iter()
                .map(|value| common_types::Kiid { value })
                .collect(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_REFILL_ZONES))
            .await?;
        let _ = response_payload_as_any(response, RES_PROTOBUF_EMPTY)?;
        Ok(())
    }
    /// Returns pad polygon responses as raw protobuf payloads.
    pub async fn get_pad_shape_as_polygon_raw(
        &self,
        pad_ids: Vec<String>,
        layer_id: i32,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        if pad_ids.is_empty() {
            return Ok(Vec::new());
        }

        let board = self.current_board_document_proto().await?;
        let mut payloads = Vec::new();
        for chunk in pad_ids.chunks(PAD_QUERY_CHUNK_SIZE) {
            let command = board_commands::GetPadShapeAsPolygon {
                board: Some(board.clone()),
                pads: chunk
                    .iter()
                    .cloned()
                    .map(|value| common_types::Kiid { value })
                    .collect(),
                layer: layer_id,
            };

            let response = self
                .send_command(envelope::pack_any(&command, CMD_GET_PAD_SHAPE_AS_POLYGON))
                .await?;
            payloads.push(response_payload_as_any(
                response,
                RES_PAD_SHAPE_AS_POLYGON_RESPONSE,
            )?);
        }

        Ok(payloads)
    }

    /// Returns mapped pad polygons for the requested layer.
    pub async fn get_pad_shape_as_polygon(
        &self,
        pad_ids: Vec<String>,
        layer_id: i32,
    ) -> Result<Vec<PadShapeAsPolygonEntry>, KiCadError> {
        if pad_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let layer_name = layer_to_model(layer_id).name;

        let payloads = self.get_pad_shape_as_polygon_raw(pad_ids, layer_id).await?;
        for payload in payloads {
            let payload: board_commands::PadShapeAsPolygonResponse =
                decode_any(&payload, RES_PAD_SHAPE_AS_POLYGON_RESPONSE)?;

            if payload.pads.len() != payload.polygons.len() {
                return Err(KiCadError::InvalidResponse {
                    reason: format!(
                        "GetPadShapeAsPolygon returned mismatched arrays: pads={}, polygons={}",
                        payload.pads.len(),
                        payload.polygons.len()
                    ),
                });
            }

            for (pad, polygon) in payload.pads.into_iter().zip(payload.polygons.into_iter()) {
                entries.push(PadShapeAsPolygonEntry {
                    pad_id: pad.value,
                    layer_id,
                    layer_name: layer_name.clone(),
                    polygon: map_polygon_with_holes(polygon)?,
                });
            }
        }

        Ok(entries)
    }

    /// Returns padstack presence responses as raw protobuf payloads.
    pub async fn check_padstack_presence_on_layers_raw(
        &self,
        item_ids: Vec<String>,
        layer_ids: Vec<i32>,
    ) -> Result<Vec<prost_types::Any>, KiCadError> {
        if item_ids.is_empty() || layer_ids.is_empty() {
            return Ok(Vec::new());
        }

        let board = self.current_board_document_proto().await?;
        let mut payloads = Vec::new();
        for chunk in item_ids.chunks(PAD_QUERY_CHUNK_SIZE) {
            let command = board_commands::CheckPadstackPresenceOnLayers {
                board: Some(board.clone()),
                items: chunk
                    .iter()
                    .cloned()
                    .map(|value| common_types::Kiid { value })
                    .collect(),
                layers: layer_ids.clone(),
            };
            let response = self
                .send_command(envelope::pack_any(
                    &command,
                    CMD_CHECK_PADSTACK_PRESENCE_ON_LAYERS,
                ))
                .await?;
            payloads.push(response_payload_as_any(
                response,
                RES_PADSTACK_PRESENCE_RESPONSE,
            )?);
        }

        Ok(payloads)
    }

    /// Returns mapped padstack presence for item and layer combinations.
    pub async fn check_padstack_presence_on_layers(
        &self,
        item_ids: Vec<String>,
        layer_ids: Vec<i32>,
    ) -> Result<Vec<PadstackPresenceEntry>, KiCadError> {
        if item_ids.is_empty() || layer_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let payloads = self
            .check_padstack_presence_on_layers_raw(item_ids, layer_ids)
            .await?;
        for payload in payloads {
            let payload: board_commands::PadstackPresenceResponse =
                decode_any(&payload, RES_PADSTACK_PRESENCE_RESPONSE)?;
            for row in payload.entries {
                let item = row.item.ok_or_else(|| KiCadError::InvalidResponse {
                    reason: "PadstackPresenceEntry missing item id".to_string(),
                })?;

                let layer = layer_to_model(row.layer);
                let presence = map_padstack_presence(row.presence);

                entries.push(PadstackPresenceEntry {
                    item_id: item.value,
                    layer_id: row.layer,
                    layer_name: layer.name,
                    presence,
                });
            }
        }

        Ok(entries)
    }

    /// Returns axis-aligned bounding boxes for item ids.
    pub async fn get_item_bounding_boxes(
        &self,
        item_ids: Vec<String>,
        include_child_text: bool,
    ) -> Result<Vec<ItemBoundingBox>, KiCadError> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mode = if include_child_text {
            common_commands::BoundingBoxMode::BbmItemAndChildText
        } else {
            common_commands::BoundingBoxMode::BbmItemOnly
        };

        let command = common_commands::GetBoundingBox {
            header: Some(self.current_board_item_header().await?),
            items: item_ids
                .into_iter()
                .map(|id| common_types::Kiid { value: id })
                .collect(),
            mode: mode as i32,
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_BOUNDING_BOX))
            .await?;

        let payload: common_commands::GetBoundingBoxResponse =
            envelope::unpack_any(&response, RES_GET_BOUNDING_BOX_RESPONSE)?;

        map_item_bounding_boxes(payload.items, payload.boxes)
    }

    /// Runs hit-test for a specific item at a position with tolerance.
    pub async fn hit_test_item(
        &self,
        item_id: String,
        position: Vector2Nm,
        tolerance_nm: i32,
    ) -> Result<ItemHitTestResult, KiCadError> {
        let command = common_commands::HitTest {
            header: Some(self.current_board_item_header().await?),
            id: Some(common_types::Kiid { value: item_id }),
            position: Some(common_types::Vector2 {
                x_nm: position.x_nm,
                y_nm: position.y_nm,
            }),
            tolerance: tolerance_nm,
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_HIT_TEST))
            .await?;

        let payload: common_commands::HitTestResponse =
            envelope::unpack_any(&response, RES_HIT_TEST_RESPONSE)?;

        Ok(map_hit_test_result(payload.result))
    }
}
