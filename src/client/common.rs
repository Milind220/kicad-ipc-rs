//! Common API operations: version, paths, documents, text variables, and text geometry.

use crate::envelope;
use crate::error::KiCadError;
use crate::model::board::*;
use crate::model::common::*;
use crate::proto::kiapi::common::commands as common_commands;
use crate::proto::kiapi::common::project as common_project;
use crate::proto::kiapi::common::types as common_types;
use std::collections::BTreeMap;
use std::path::PathBuf;

use super::mappers::*;
use super::{
    is_get_open_documents_unhandled, map_document_specifier, model_document_to_proto,
    project_document_proto, resolve_current_project_path, rpc, select_single_board_document,
    select_single_project_path, KiCadClient, CMD_EXPAND_TEXT_VARIABLES, CMD_GET_KICAD_BINARY_PATH,
    CMD_GET_NET_CLASSES, CMD_GET_OPEN_DOCUMENTS, CMD_GET_PLUGIN_SETTINGS_PATH,
    CMD_GET_TEXT_AS_SHAPES, CMD_GET_TEXT_EXTENTS, CMD_GET_TEXT_VARIABLES, CMD_GET_VERSION,
    CMD_PING, CMD_REFRESH_EDITOR, CMD_RUN_ACTION, CMD_SET_NET_CLASSES, CMD_SET_TEXT_VARIABLES,
    RES_BOX2, RES_EXPAND_TEXT_VARIABLES_RESPONSE, RES_GET_OPEN_DOCUMENTS,
    RES_GET_TEXT_AS_SHAPES_RESPONSE, RES_GET_VERSION, RES_NET_CLASSES_RESPONSE, RES_PATH_RESPONSE,
    RES_PROTOBUF_EMPTY, RES_RUN_ACTION_RESPONSE, RES_STRING_RESPONSE, RES_TEXT_VARIABLES,
};

impl KiCadClient {
    pub async fn ping(&self) -> Result<(), KiCadError> {
        let command = envelope::pack_any(&common_commands::Ping {}, CMD_PING);
        self.send_command(command).await?;
        Ok(())
    }

    /// Requests KiCad to refresh a specific editor frame.
    pub async fn refresh_editor(&self, frame: EditorFrameType) -> Result<(), KiCadError> {
        let command = envelope::pack_any(
            &common_commands::RefreshEditor {
                frame: frame.to_proto(),
            },
            CMD_REFRESH_EDITOR,
        );
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn run_action_raw(
        &self,
        action: impl Into<String>,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::RunAction {
            action: action.into(),
        };
        rpc!(self, CMD_RUN_ACTION, command, RES_RUN_ACTION_RESPONSE)
    }
    /// Runs a KiCad action by action name and returns mapped status.
    pub async fn run_action(
        &self,
        action: impl Into<String>,
    ) -> Result<RunActionStatus, KiCadError> {
        let payload = self.run_action_raw(action).await?;
        let response: common_commands::RunActionResponse =
            decode_any(&payload, RES_RUN_ACTION_RESPONSE)?;
        Ok(map_run_action_status(response.status))
    }

    /// Queries KiCad version info for the connected instance.
    pub async fn get_version(&self) -> Result<VersionInfo, KiCadError> {
        let command = envelope::pack_any(&common_commands::GetVersion {}, CMD_GET_VERSION);
        let response = self.send_command(command).await?;

        let payload: common_commands::GetVersionResponse =
            envelope::unpack_any(&response, RES_GET_VERSION)?;

        let version = payload.version.ok_or_else(|| KiCadError::MissingPayload {
            expected_type_url: "kiapi.common.types.KiCadVersion".to_string(),
        })?;

        Ok(VersionInfo {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            full_version: version.full_version,
        })
    }

    pub async fn get_kicad_binary_path_raw(
        &self,
        binary_name: impl Into<String>,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::GetKiCadBinaryPath {
            binary_name: binary_name.into(),
        };
        rpc!(self, CMD_GET_KICAD_BINARY_PATH, command, RES_PATH_RESPONSE)
    }
    /// Resolves a KiCad binary path by binary name.
    pub async fn get_kicad_binary_path(
        &self,
        binary_name: impl Into<String>,
    ) -> Result<String, KiCadError> {
        let payload = self.get_kicad_binary_path_raw(binary_name).await?;
        let response: common_commands::PathResponse = decode_any(&payload, RES_PATH_RESPONSE)?;
        Ok(response.path)
    }

    pub async fn get_plugin_settings_path_raw(
        &self,
        identifier: impl Into<String>,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::GetPluginSettingsPath {
            identifier: identifier.into(),
        };
        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_PLUGIN_SETTINGS_PATH))
            .await?;
        response_payload_as_any(response, RES_STRING_RESPONSE)
    }

    /// Resolves plugin settings path for a plugin identifier.
    pub async fn get_plugin_settings_path(
        &self,
        identifier: impl Into<String>,
    ) -> Result<String, KiCadError> {
        let payload = self.get_plugin_settings_path_raw(identifier).await?;
        let response: common_commands::StringResponse = decode_any(&payload, RES_STRING_RESPONSE)?;
        Ok(response.response)
    }

    /// Lists open KiCad documents of the requested type.
    pub async fn get_open_documents(
        &self,
        document_type: DocumentType,
    ) -> Result<Vec<DocumentSpecifier>, KiCadError> {
        let command = common_commands::GetOpenDocuments {
            r#type: document_type.to_proto(),
        };

        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_OPEN_DOCUMENTS))
            .await?;

        let payload: common_commands::GetOpenDocumentsResponse =
            envelope::unpack_any(&response, RES_GET_OPEN_DOCUMENTS)?;

        Ok(payload
            .documents
            .into_iter()
            .filter_map(map_document_specifier)
            .collect())
    }

    pub async fn get_net_classes_raw(&self) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::GetNetClasses {};
        rpc!(self, CMD_GET_NET_CLASSES, command, RES_NET_CLASSES_RESPONSE)
    }
    /// Reads project net classes from the current project context.
    pub async fn get_net_classes(&self) -> Result<Vec<NetClassInfo>, KiCadError> {
        let payload = self.get_net_classes_raw().await?;
        let response: common_commands::NetClassesResponse =
            decode_any(&payload, RES_NET_CLASSES_RESPONSE)?;

        let mut classes: Vec<NetClassInfo> = response
            .net_classes
            .into_iter()
            .map(map_net_class_info)
            .collect();
        classes.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(classes)
    }

    pub async fn set_net_classes_raw(
        &self,
        net_classes: Vec<NetClassInfo>,
        merge_mode: MapMergeMode,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::SetNetClasses {
            net_classes: net_classes
                .into_iter()
                .map(net_class_info_to_proto)
                .collect(),
            merge_mode: map_merge_mode_to_proto(merge_mode),
        };
        let response = self
            .send_command(envelope::pack_any(&command, CMD_SET_NET_CLASSES))
            .await?;
        response_payload_as_any(response, RES_PROTOBUF_EMPTY)
    }

    /// Replaces or merges project net classes, then returns current classes.
    pub async fn set_net_classes(
        &self,
        net_classes: Vec<NetClassInfo>,
        merge_mode: MapMergeMode,
    ) -> Result<Vec<NetClassInfo>, KiCadError> {
        let _ = self.set_net_classes_raw(net_classes, merge_mode).await?;
        self.get_net_classes().await
    }

    pub async fn get_text_variables_raw(&self) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::GetTextVariables {
            document: Some(project_document_proto()),
        };
        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_TEXT_VARIABLES))
            .await?;
        response_payload_as_any(response, RES_TEXT_VARIABLES)
    }

    /// Reads project text variables.
    pub async fn get_text_variables(&self) -> Result<BTreeMap<String, String>, KiCadError> {
        let payload = self.get_text_variables_raw().await?;
        let response: common_project::TextVariables = decode_any(&payload, RES_TEXT_VARIABLES)?;
        Ok(response.variables.into_iter().collect())
    }

    pub async fn set_text_variables_raw(
        &self,
        variables: BTreeMap<String, String>,
        merge_mode: MapMergeMode,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::SetTextVariables {
            document: Some(project_document_proto()),
            variables: Some(common_project::TextVariables {
                variables: variables.into_iter().collect(),
            }),
            merge_mode: map_merge_mode_to_proto(merge_mode),
        };
        let response = self
            .send_command(envelope::pack_any(&command, CMD_SET_TEXT_VARIABLES))
            .await?;
        response_payload_as_any(response, RES_PROTOBUF_EMPTY)
    }

    /// Replaces or merges project text variables, then returns current values.
    pub async fn set_text_variables(
        &self,
        variables: BTreeMap<String, String>,
        merge_mode: MapMergeMode,
    ) -> Result<BTreeMap<String, String>, KiCadError> {
        let _ = self.set_text_variables_raw(variables, merge_mode).await?;
        self.get_text_variables().await
    }

    pub async fn expand_text_variables_raw(
        &self,
        text: Vec<String>,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::ExpandTextVariables {
            document: Some(project_document_proto()),
            text,
        };
        let response = self
            .send_command(envelope::pack_any(&command, CMD_EXPAND_TEXT_VARIABLES))
            .await?;
        response_payload_as_any(response, RES_EXPAND_TEXT_VARIABLES_RESPONSE)
    }

    /// Expands `${VAR}`-style text variables using current project context.
    pub async fn expand_text_variables(
        &self,
        text: Vec<String>,
    ) -> Result<Vec<String>, KiCadError> {
        let payload = self.expand_text_variables_raw(text).await?;
        let response: common_commands::ExpandTextVariablesResponse =
            decode_any(&payload, RES_EXPAND_TEXT_VARIABLES_RESPONSE)?;
        Ok(response.text)
    }

    pub async fn get_text_extents_raw(
        &self,
        text: TextSpec,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::GetTextExtents {
            text: Some(text_spec_to_proto(text)),
        };
        rpc!(self, CMD_GET_TEXT_EXTENTS, command, RES_BOX2)
    }
    /// Computes rendered text extents in nanometer units.
    pub async fn get_text_extents(&self, text: TextSpec) -> Result<TextExtents, KiCadError> {
        let payload = self.get_text_extents_raw(text).await?;
        let response: common_types::Box2 = decode_any(&payload, RES_BOX2)?;
        let position = response
            .position
            .ok_or_else(|| KiCadError::InvalidResponse {
                reason: "GetTextExtents response missing position".to_string(),
            })?;
        let size = response.size.ok_or_else(|| KiCadError::InvalidResponse {
            reason: "GetTextExtents response missing size".to_string(),
        })?;

        Ok(TextExtents {
            x_nm: position.x_nm,
            y_nm: position.y_nm,
            width_nm: size.x_nm,
            height_nm: size.y_nm,
        })
    }

    pub async fn get_text_as_shapes_raw(
        &self,
        text: Vec<TextObjectSpec>,
    ) -> Result<prost_types::Any, KiCadError> {
        let command = common_commands::GetTextAsShapes {
            text: text.into_iter().map(text_object_spec_to_proto).collect(),
        };
        let response = self
            .send_command(envelope::pack_any(&command, CMD_GET_TEXT_AS_SHAPES))
            .await?;
        response_payload_as_any(response, RES_GET_TEXT_AS_SHAPES_RESPONSE)
    }

    /// Converts text/textbox specs into drawable shape geometry.
    pub async fn get_text_as_shapes(
        &self,
        text: Vec<TextObjectSpec>,
    ) -> Result<Vec<TextAsShapesEntry>, KiCadError> {
        let payload = self.get_text_as_shapes_raw(text).await?;
        let response: common_commands::GetTextAsShapesResponse =
            decode_any(&payload, RES_GET_TEXT_AS_SHAPES_RESPONSE)?;

        response
            .text_with_shapes
            .into_iter()
            .map(map_text_with_shapes)
            .collect()
    }

    /// Returns the current project path.
    ///
    /// First queries open PCB documents. If KiCad reports `GetOpenDocuments` as unhandled,
    /// this falls back to the `KIPRJMOD` environment variable when available.
    pub async fn get_current_project_path(&self) -> Result<PathBuf, KiCadError> {
        let docs = self.get_open_documents(DocumentType::Pcb).await;
        resolve_current_project_path(docs)
    }

    /// Returns `true` when at least one PCB document is open in KiCad.
    pub async fn has_open_board(&self) -> Result<bool, KiCadError> {
        let docs = self.get_open_documents(DocumentType::Pcb).await?;
        Ok(!docs.is_empty())
    }
}
