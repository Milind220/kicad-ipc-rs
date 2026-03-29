use crate::envelope;
use crate::error::KiCadError;
use crate::model::board::*;
use crate::model::common::*;
use crate::proto::kiapi::board::commands as board_commands;
use crate::proto::kiapi::common::types as common_types;

use super::mappers::*;
use super::{
    KiCadClient, CMD_GET_ACTIVE_LAYER, CMD_GET_BOARD_EDITOR_APPEARANCE_SETTINGS,
    CMD_GET_BOARD_ENABLED_LAYERS, CMD_GET_BOARD_LAYER_NAME, CMD_GET_BOARD_ORIGIN,
    CMD_GET_BOARD_STACKUP, CMD_GET_GRAPHICS_DEFAULTS, CMD_GET_NETS, CMD_GET_VISIBLE_LAYERS,
    CMD_INJECT_DRC_ERROR, CMD_INTERACTIVE_MOVE_ITEMS, CMD_SET_ACTIVE_LAYER,
    CMD_SET_BOARD_EDITOR_APPEARANCE_SETTINGS, CMD_SET_BOARD_ENABLED_LAYERS, CMD_SET_BOARD_ORIGIN,
    CMD_SET_VISIBLE_LAYERS, CMD_UPDATE_BOARD_STACKUP, RES_BOARD_EDITOR_APPEARANCE_SETTINGS,
    RES_BOARD_LAYERS, RES_BOARD_LAYER_NAME_RESPONSE, RES_BOARD_LAYER_RESPONSE,
    RES_BOARD_STACKUP_RESPONSE, RES_GET_BOARD_ENABLED_LAYERS, RES_GET_NETS,
    RES_GRAPHICS_DEFAULTS_RESPONSE, RES_INJECT_DRC_ERROR_RESPONSE, RES_PROTOBUF_EMPTY, RES_VECTOR2,
};

impl KiCadClient {
    pub async fn get_nets(&self) -> Result<Vec<BoardNet>, KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::GetNets {
            board: Some(board),
            netclass_filter: Vec::new(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_NETS))
            .await?;

        let payload: board_commands::NetsResponse = envelope::unpack_any(&response, RES_GET_NETS)?;

        Ok(payload
            .nets
            .into_iter()
            .map(|net| BoardNet {
                code: net.code.map_or(0, |code| code.value),
                name: net.name,
            })
            .collect())
    }

    pub async fn get_board_enabled_layers(&self) -> Result<BoardEnabledLayers, KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::GetBoardEnabledLayers { board: Some(board) };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_BOARD_ENABLED_LAYERS))
            .await?;

        let payload: board_commands::BoardEnabledLayersResponse =
            envelope::unpack_any(&response, RES_GET_BOARD_ENABLED_LAYERS)?;

        Ok(map_board_enabled_layers_response(payload))
    }

    pub async fn set_board_enabled_layers(
        &self,
        copper_layer_count: u32,
        layer_ids: Vec<i32>,
    ) -> Result<BoardEnabledLayers, KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::SetBoardEnabledLayers {
            board: Some(board),
            copper_layer_count,
            layers: layer_ids,
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_SET_BOARD_ENABLED_LAYERS))
            .await?;

        let payload: board_commands::BoardEnabledLayersResponse =
            envelope::unpack_any(&response, RES_GET_BOARD_ENABLED_LAYERS)?;
        Ok(map_board_enabled_layers_response(payload))
    }

    pub async fn get_active_layer(&self) -> Result<BoardLayerInfo, KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::GetActiveLayer { board: Some(board) };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_ACTIVE_LAYER))
            .await?;

        let payload: board_commands::BoardLayerResponse =
            envelope::unpack_any(&response, RES_BOARD_LAYER_RESPONSE)?;

        Ok(layer_to_model(payload.layer))
    }

    pub async fn set_active_layer(&self, layer_id: i32) -> Result<(), KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::SetActiveLayer {
            board: Some(board),
            layer: layer_id,
        };

        self.send_command(envelope::pack_any(&command, CMD_SET_ACTIVE_LAYER))
            .await?;
        Ok(())
    }

    pub async fn get_visible_layers(&self) -> Result<Vec<BoardLayerInfo>, KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::GetVisibleLayers { board: Some(board) };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_VISIBLE_LAYERS))
            .await?;

        let payload: board_commands::BoardLayers =
            envelope::unpack_any(&response, RES_BOARD_LAYERS)?;

        Ok(payload.layers.into_iter().map(layer_to_model).collect())
    }

    pub async fn set_visible_layers(&self, layer_ids: Vec<i32>) -> Result<(), KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::SetVisibleLayers {
            board: Some(board),
            layers: layer_ids,
        };

        self.send_command(envelope::pack_any(&command, CMD_SET_VISIBLE_LAYERS))
            .await?;
        Ok(())
    }

    pub async fn get_board_layer_name(&self, layer_id: i32) -> Result<String, KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::GetBoardLayerName {
            board: Some(board),
            layer: layer_id,
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_BOARD_LAYER_NAME))
            .await?;

        let payload: board_commands::BoardLayerNameResponse =
            envelope::unpack_any(&response, RES_BOARD_LAYER_NAME_RESPONSE)?;
        Ok(payload.name)
    }

    pub async fn get_board_origin(&self, kind: BoardOriginKind) -> Result<Vector2Nm, KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::GetBoardOrigin {
            board: Some(board),
            r#type: board_origin_kind_to_proto(kind),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_BOARD_ORIGIN))
            .await?;

        let payload: common_types::Vector2 = envelope::unpack_any(&response, RES_VECTOR2)?;
        Ok(Vector2Nm {
            x_nm: payload.x_nm,
            y_nm: payload.y_nm,
        })
    }

    pub async fn set_board_origin(
        &self,
        kind: BoardOriginKind,
        origin: Vector2Nm,
    ) -> Result<(), KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::SetBoardOrigin {
            board: Some(board),
            r#type: board_origin_kind_to_proto(kind),
            origin: Some(vector2_nm_to_proto(origin)),
        };

        self.send_command(envelope::pack_any(&command, CMD_SET_BOARD_ORIGIN))
            .await?;
        Ok(())
    }

    /// Returns a compact summary of the current PCB selection.

    pub async fn inject_drc_error_raw(
        &self,
        severity: DrcSeverity,
        message: impl Into<String>,
        position: Option<Vector2Nm>,
        item_ids: Vec<String>,
    ) -> Result<prost_types::Any, KiCadError> {
        let board = self.current_board_document_proto().await?;
        let command = board_commands::InjectDrcError {
            board: Some(board),
            severity: drc_severity_to_proto(severity),
            message: message.into(),
            position: position.map(vector2_nm_to_proto),
            items: item_ids
                .into_iter()
                .map(|value| common_types::Kiid { value })
                .collect(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_INJECT_DRC_ERROR))
            .await?;
        response_payload_as_any(response, RES_INJECT_DRC_ERROR_RESPONSE)
    }

    pub async fn inject_drc_error(
        &self,
        severity: DrcSeverity,
        message: impl Into<String>,
        position: Option<Vector2Nm>,
        item_ids: Vec<String>,
    ) -> Result<Option<String>, KiCadError> {
        let payload = self
            .inject_drc_error_raw(severity, message, position, item_ids)
            .await?;
        let response: board_commands::InjectDrcErrorResponse =
            decode_any(&payload, RES_INJECT_DRC_ERROR_RESPONSE)?;
        Ok(response.marker.map(|marker| marker.value))
    }

    pub async fn get_board_stackup_raw(&self) -> Result<prost_types::Any, KiCadError> {
        let command = board_commands::GetBoardStackup {
            board: Some(self.current_board_document_proto().await?),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_BOARD_STACKUP))
            .await?;

        response_payload_as_any(response, RES_BOARD_STACKUP_RESPONSE)
    }

    /// Reads board stackup from the active PCB document.
    pub async fn get_board_stackup(&self) -> Result<BoardStackup, KiCadError> {
        let payload = self.get_board_stackup_raw().await?;
        let response: board_commands::BoardStackupResponse =
            decode_any(&payload, RES_BOARD_STACKUP_RESPONSE)?;
        Ok(map_board_stackup(response.stackup.unwrap_or_default()))
    }

    pub async fn update_board_stackup_raw(
        &self,
        stackup: BoardStackup,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = board_commands::UpdateBoardStackup {
            board: Some(self.current_board_document_proto().await?),
            stackup: Some(board_stackup_to_proto(stackup)),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_UPDATE_BOARD_STACKUP))
            .await?;

        response_payload_as_any(response, RES_BOARD_STACKUP_RESPONSE)
    }

    /// Writes a board stackup and returns KiCad's resulting stackup state.
    pub async fn update_board_stackup(
        &self,
        stackup: BoardStackup,
    ) -> Result<BoardStackup, KiCadError> {
        let payload = self.update_board_stackup_raw(stackup).await?;
        let response: board_commands::BoardStackupResponse =
            decode_any(&payload, RES_BOARD_STACKUP_RESPONSE)?;
        Ok(map_board_stackup(response.stackup.unwrap_or_default()))
    }

    pub async fn get_graphics_defaults_raw(&self) -> Result<prost_types::Any, KiCadError> {
        let command = board_commands::GetGraphicsDefaults {
            board: Some(self.current_board_document_proto().await?),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_GRAPHICS_DEFAULTS))
            .await?;

        response_payload_as_any(response, RES_GRAPHICS_DEFAULTS_RESPONSE)
    }

    pub async fn get_graphics_defaults(&self) -> Result<GraphicsDefaults, KiCadError> {
        let payload = self.get_graphics_defaults_raw().await?;
        let response: board_commands::GraphicsDefaultsResponse =
            decode_any(&payload, RES_GRAPHICS_DEFAULTS_RESPONSE)?;
        Ok(map_graphics_defaults(response.defaults.unwrap_or_default()))
    }

    pub async fn get_board_editor_appearance_settings_raw(
        &self,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = board_commands::GetBoardEditorAppearanceSettings {};

        let response = self
            .send_command(envelope::pack_any(
                &command,
                CMD_GET_BOARD_EDITOR_APPEARANCE_SETTINGS,
            ))
            .await?;

        response_payload_as_any(response, RES_BOARD_EDITOR_APPEARANCE_SETTINGS)
    }

    pub async fn get_board_editor_appearance_settings(
        &self,
    ) -> Result<BoardEditorAppearanceSettings, KiCadError> {
        let payload = self.get_board_editor_appearance_settings_raw().await?;
        let response: board_commands::BoardEditorAppearanceSettings =
            decode_any(&payload, RES_BOARD_EDITOR_APPEARANCE_SETTINGS)?;
        Ok(map_board_editor_appearance_settings(response))
    }

    pub async fn set_board_editor_appearance_settings(
        &self,
        settings: BoardEditorAppearanceSettings,
    ) -> Result<BoardEditorAppearanceSettings, KiCadError> {
        let command = board_commands::SetBoardEditorAppearanceSettings {
            settings: Some(board_editor_appearance_settings_to_proto(settings)),
        };

        let response = self
            .send_command(envelope::pack_any(
                &command,
                CMD_SET_BOARD_EDITOR_APPEARANCE_SETTINGS,
            ))
            .await?;
        let _ = response_payload_as_any(response, RES_PROTOBUF_EMPTY)?;
        self.get_board_editor_appearance_settings().await
    }

    pub async fn interactive_move_items_raw(
        &self,
        item_ids: Vec<String>,
    ) -> Result<prost_types::Any, KiCadError> {
        if item_ids.is_empty() {
            return Err(KiCadError::Config {
                reason: "interactive_move_items_raw requires at least one item id".to_string(),
            });
        }

        let command = board_commands::InteractiveMoveItems {
            board: Some(self.current_board_document_proto().await?),
            items: item_ids
                .into_iter()
                .map(|value| common_types::Kiid { value })
                .collect(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_INTERACTIVE_MOVE_ITEMS))
            .await?;
        response_payload_as_any(response, RES_PROTOBUF_EMPTY)
    }

    pub async fn interactive_move_items(&self, item_ids: Vec<String>) -> Result<(), KiCadError> {
        let _ = self.interactive_move_items_raw(item_ids).await?;
        Ok(())
    }
}
