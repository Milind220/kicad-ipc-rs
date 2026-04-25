use crate::envelope;
use crate::error::KiCadError;
use crate::model::board::Vector2Nm;
use crate::pcb_item_types;
use crate::proto::kiapi::{board::types as board_types, common::types as common_types};
use prost::Message;
use prost_types::Any;

/// Typed kind classification for editable PCB item payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditablePcbItemKind {
    /// A board track segment.
    Track,
    /// A board arc track segment.
    Arc,
    /// A via item.
    Via,
    /// A placed footprint instance.
    Footprint,
    /// A footprint pad.
    Pad,
    /// A board graphic shape.
    BoardGraphicShape,
    /// A board text item.
    BoardText,
    /// A board text box item.
    BoardTextBox,
    /// A footprint field item.
    Field,
    /// A board zone.
    Zone,
    /// A board dimension.
    Dimension,
    /// A board group.
    Group,
    /// A board reference image.
    ReferenceImage,
    /// A payload whose type URL is not recognized by this crate version.
    Unknown,
}

/// Layer presence model for editable PCB items.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayerSet {
    /// Item belongs to exactly one board layer id.
    Single(i32),
    /// Item belongs to multiple board layer ids.
    Multi(Vec<i32>),
    /// Item layer membership is represented by a padstack definition.
    Padstack,
    /// Item has no direct layer relationship.
    None,
}

/// Raw editable payload wrapper for unknown/unsupported item types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawPcbItem {
    /// Original protobuf payload.
    pub raw: Any,
}

macro_rules! impl_proto_wrapper {
    ($name:ident, $proto:ty) => {
        #[derive(Clone, Debug, PartialEq)]
        /// Typed editable PCB item wrapper preserving the full protobuf payload.
        pub struct $name {
            proto: $proto,
        }

        impl $name {
            /// Wraps a decoded protobuf item.
            pub fn from_proto(proto: $proto) -> Self {
                Self { proto }
            }

            /// Returns the underlying protobuf payload.
            pub fn proto(&self) -> &$proto {
                &self.proto
            }

            /// Returns a mutable reference to the underlying protobuf payload.
            pub fn proto_mut(&mut self) -> &mut $proto {
                &mut self.proto
            }

            /// Consumes this wrapper and returns the underlying protobuf payload.
            pub fn into_proto(self) -> $proto {
                self.proto
            }
        }
    };
}

impl_proto_wrapper!(TrackItem, board_types::Track);
impl_proto_wrapper!(ArcItem, board_types::Arc);
impl_proto_wrapper!(ViaItem, board_types::Via);
impl_proto_wrapper!(FootprintItem, board_types::FootprintInstance);
impl_proto_wrapper!(PadItem, board_types::Pad);
impl_proto_wrapper!(BoardGraphicShapeItem, board_types::BoardGraphicShape);
impl_proto_wrapper!(BoardTextItem, board_types::BoardText);
impl_proto_wrapper!(BoardTextBoxItem, board_types::BoardTextBox);
impl_proto_wrapper!(FieldItem, board_types::Field);
impl_proto_wrapper!(ZoneItem, board_types::Zone);
impl_proto_wrapper!(DimensionItem, board_types::Dimension);
impl_proto_wrapper!(GroupItem, board_types::Group);
impl_proto_wrapper!(ReferenceImageItem, board_types::ReferenceImage);

impl TrackItem {
    /// Returns the track KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }

    /// Returns the track layer id.
    pub fn layer_id(&self) -> i32 {
        self.proto.layer
    }

    /// Sets the track layer id.
    pub fn set_layer_id(&mut self, layer_id: i32) {
        self.proto.layer = layer_id;
    }

    /// Returns the start point in nanometers, if present.
    pub fn start_nm(&self) -> Option<Vector2Nm> {
        self.proto.start.map(vector2_from_proto)
    }

    /// Returns the end point in nanometers, if present.
    pub fn end_nm(&self) -> Option<Vector2Nm> {
        self.proto.end.map(vector2_from_proto)
    }

    /// Sets the start point.
    pub fn set_start_nm(&mut self, value: Vector2Nm) {
        self.proto.start = Some(vector2_to_proto(value));
    }

    /// Sets the end point.
    pub fn set_end_nm(&mut self, value: Vector2Nm) {
        self.proto.end = Some(vector2_to_proto(value));
    }
}

impl ArcItem {
    /// Returns the arc KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }

    /// Returns the arc layer id.
    pub fn layer_id(&self) -> i32 {
        self.proto.layer
    }

    /// Sets the arc layer id.
    pub fn set_layer_id(&mut self, layer_id: i32) {
        self.proto.layer = layer_id;
    }

    /// Returns the start point in nanometers, if present.
    pub fn start_nm(&self) -> Option<Vector2Nm> {
        self.proto.start.map(vector2_from_proto)
    }

    /// Returns the midpoint in nanometers, if present.
    pub fn mid_nm(&self) -> Option<Vector2Nm> {
        self.proto.mid.map(vector2_from_proto)
    }

    /// Returns the end point in nanometers, if present.
    pub fn end_nm(&self) -> Option<Vector2Nm> {
        self.proto.end.map(vector2_from_proto)
    }

    /// Sets the start point.
    pub fn set_start_nm(&mut self, value: Vector2Nm) {
        self.proto.start = Some(vector2_to_proto(value));
    }

    /// Sets the midpoint.
    pub fn set_mid_nm(&mut self, value: Vector2Nm) {
        self.proto.mid = Some(vector2_to_proto(value));
    }

    /// Sets the end point.
    pub fn set_end_nm(&mut self, value: Vector2Nm) {
        self.proto.end = Some(vector2_to_proto(value));
    }
}

impl ViaItem {
    /// Returns the via KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }

    /// Returns the via position in nanometers, if present.
    pub fn position_nm(&self) -> Option<Vector2Nm> {
        self.proto.position.map(vector2_from_proto)
    }

    /// Sets via position.
    pub fn set_position_nm(&mut self, value: Vector2Nm) {
        self.proto.position = Some(vector2_to_proto(value));
    }
}

impl FootprintItem {
    /// Returns the footprint KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }

    /// Returns the footprint layer id.
    pub fn layer_id(&self) -> i32 {
        self.proto.layer
    }

    /// Sets the footprint layer id.
    pub fn set_layer_id(&mut self, layer_id: i32) {
        self.proto.layer = layer_id;
    }
}

impl PadItem {
    /// Returns the pad KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }
}

impl BoardGraphicShapeItem {
    /// Returns the shape KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }

    /// Returns the shape layer id.
    pub fn layer_id(&self) -> i32 {
        self.proto.layer
    }

    /// Sets the shape layer id.
    pub fn set_layer_id(&mut self, layer_id: i32) {
        self.proto.layer = layer_id;
    }
}

impl BoardTextItem {
    /// Returns the text KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }

    /// Returns the text layer id.
    pub fn layer_id(&self) -> i32 {
        self.proto.layer
    }

    /// Sets the text layer id.
    pub fn set_layer_id(&mut self, layer_id: i32) {
        self.proto.layer = layer_id;
    }
}

impl BoardTextBoxItem {
    /// Returns the text box KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }

    /// Returns the text box layer id.
    pub fn layer_id(&self) -> i32 {
        self.proto.layer
    }

    /// Sets the text box layer id.
    pub fn set_layer_id(&mut self, layer_id: i32) {
        self.proto.layer = layer_id;
    }
}

impl FieldItem {
    /// Returns the field id value, if present.
    pub fn field_id(&self) -> Option<i32> {
        self.proto.id.map(|id| id.id)
    }

    /// Returns nested board text, if present.
    pub fn text(&self) -> Option<&board_types::BoardText> {
        self.proto.text.as_ref()
    }

    /// Returns mutable nested board text, if present.
    pub fn text_mut(&mut self) -> Option<&mut board_types::BoardText> {
        self.proto.text.as_mut()
    }

    /// Sets the layer id on the nested board text.
    pub fn set_layer_id(&mut self, layer_id: i32) -> Result<(), KiCadError> {
        let text = self
            .proto
            .text
            .as_mut()
            .ok_or_else(|| KiCadError::InvalidResponse {
                reason: "field has no nested board text; cannot set layer".to_string(),
            })?;
        text.layer = layer_id;
        Ok(())
    }
}

impl ZoneItem {
    /// Returns the zone KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }

    /// Returns the zone layer ids.
    pub fn layer_ids(&self) -> &[i32] {
        &self.proto.layers
    }

    /// Replaces the zone layer ids.
    pub fn set_layer_ids(&mut self, layer_ids: Vec<i32>) {
        self.proto.layers = layer_ids;
    }
}

impl DimensionItem {
    /// Returns the dimension KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }

    /// Returns the dimension layer id.
    pub fn layer_id(&self) -> i32 {
        self.proto.layer
    }

    /// Sets the dimension layer id.
    pub fn set_layer_id(&mut self, layer_id: i32) {
        self.proto.layer = layer_id;
    }
}

impl GroupItem {
    /// Builds a new group with no id and the provided member KIID values.
    pub fn new(name: impl Into<String>, member_ids: Vec<String>) -> Self {
        Self {
            proto: board_types::Group {
                id: None,
                name: name.into(),
                items: member_ids
                    .into_iter()
                    .map(|value| common_types::Kiid { value })
                    .collect(),
            },
        }
    }

    /// Returns the group KIID value.
    pub fn id(&self) -> Option<&str> {
        kiid_value(&self.proto.id)
    }

    /// Returns member KIID values.
    pub fn member_ids(&self) -> Vec<&str> {
        self.proto
            .items
            .iter()
            .map(|item| item.value.as_str())
            .collect()
    }

    /// Returns the group name.
    pub fn name(&self) -> &str {
        &self.proto.name
    }

    /// Sets the group name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.proto.name = name.into();
    }

    /// Replaces the member KIID values.
    pub fn set_member_ids(&mut self, member_ids: Vec<String>) {
        self.proto.items = member_ids
            .into_iter()
            .map(|value| common_types::Kiid { value })
            .collect();
    }
}

/// Ergonomic typed editable board item.
#[derive(Clone, Debug, PartialEq)]
pub enum EditablePcbItem {
    /// Track item.
    Track(TrackItem),
    /// Arc item.
    Arc(ArcItem),
    /// Via item.
    Via(ViaItem),
    /// Footprint instance item.
    Footprint(FootprintItem),
    /// Pad item.
    Pad(PadItem),
    /// Board graphic shape item.
    BoardGraphicShape(BoardGraphicShapeItem),
    /// Board text item.
    BoardText(BoardTextItem),
    /// Board text box item.
    BoardTextBox(BoardTextBoxItem),
    /// Field item.
    Field(FieldItem),
    /// Zone item.
    Zone(ZoneItem),
    /// Dimension item.
    Dimension(DimensionItem),
    /// Group item.
    Group(GroupItem),
    /// Reference image item.
    ReferenceImage(ReferenceImageItem),
    /// Unknown payload preserved as-is.
    Unknown(RawPcbItem),
}

impl EditablePcbItem {
    /// Decodes a raw protobuf payload into a typed editable PCB item.
    pub fn from_any(raw: Any) -> Result<Self, KiCadError> {
        let type_url = raw.type_url.clone();

        if type_url == envelope::type_url(pcb_item_types::TRACK) {
            return Ok(Self::Track(TrackItem::from_proto(decode_item(
                &raw,
                pcb_item_types::TRACK,
            )?)));
        }
        if type_url == envelope::type_url(pcb_item_types::ARC) {
            return Ok(Self::Arc(ArcItem::from_proto(decode_item(
                &raw,
                pcb_item_types::ARC,
            )?)));
        }
        if type_url == envelope::type_url(pcb_item_types::VIA) {
            return Ok(Self::Via(ViaItem::from_proto(decode_item(
                &raw,
                pcb_item_types::VIA,
            )?)));
        }
        if type_url == envelope::type_url(pcb_item_types::FOOTPRINT_INSTANCE) {
            return Ok(Self::Footprint(FootprintItem::from_proto(decode_item(
                &raw,
                pcb_item_types::FOOTPRINT_INSTANCE,
            )?)));
        }
        if type_url == envelope::type_url(pcb_item_types::PAD) {
            return Ok(Self::Pad(PadItem::from_proto(decode_item(
                &raw,
                pcb_item_types::PAD,
            )?)));
        }
        if type_url == envelope::type_url(pcb_item_types::BOARD_GRAPHIC_SHAPE) {
            return Ok(Self::BoardGraphicShape(BoardGraphicShapeItem::from_proto(
                decode_item(&raw, pcb_item_types::BOARD_GRAPHIC_SHAPE)?,
            )));
        }
        if type_url == envelope::type_url(pcb_item_types::BOARD_TEXT) {
            return Ok(Self::BoardText(BoardTextItem::from_proto(decode_item(
                &raw,
                pcb_item_types::BOARD_TEXT,
            )?)));
        }
        if type_url == envelope::type_url(pcb_item_types::BOARD_TEXT_BOX) {
            return Ok(Self::BoardTextBox(BoardTextBoxItem::from_proto(
                decode_item(&raw, pcb_item_types::BOARD_TEXT_BOX)?,
            )));
        }
        if type_url == envelope::type_url(pcb_item_types::FIELD) {
            return Ok(Self::Field(FieldItem::from_proto(decode_item(
                &raw,
                pcb_item_types::FIELD,
            )?)));
        }
        if type_url == envelope::type_url(pcb_item_types::ZONE) {
            return Ok(Self::Zone(ZoneItem::from_proto(decode_item(
                &raw,
                pcb_item_types::ZONE,
            )?)));
        }
        if type_url == envelope::type_url(pcb_item_types::DIMENSION) {
            return Ok(Self::Dimension(DimensionItem::from_proto(decode_item(
                &raw,
                pcb_item_types::DIMENSION,
            )?)));
        }
        if type_url == envelope::type_url(pcb_item_types::GROUP) {
            return Ok(Self::Group(GroupItem::from_proto(decode_item(
                &raw,
                pcb_item_types::GROUP,
            )?)));
        }
        if type_url == envelope::type_url(pcb_item_types::REFERENCE_IMAGE) {
            return Ok(Self::ReferenceImage(ReferenceImageItem::from_proto(
                decode_item(&raw, pcb_item_types::REFERENCE_IMAGE)?,
            )));
        }

        Ok(Self::Unknown(RawPcbItem { raw }))
    }

    /// Converts this editable item into a raw protobuf payload.
    pub fn into_any(self) -> Any {
        match self {
            Self::Track(item) => envelope::pack_any(&item.proto, pcb_item_types::TRACK),
            Self::Arc(item) => envelope::pack_any(&item.proto, pcb_item_types::ARC),
            Self::Via(item) => envelope::pack_any(&item.proto, pcb_item_types::VIA),
            Self::Footprint(item) => {
                envelope::pack_any(&item.proto, pcb_item_types::FOOTPRINT_INSTANCE)
            }
            Self::Pad(item) => envelope::pack_any(&item.proto, pcb_item_types::PAD),
            Self::BoardGraphicShape(item) => {
                envelope::pack_any(&item.proto, pcb_item_types::BOARD_GRAPHIC_SHAPE)
            }
            Self::BoardText(item) => envelope::pack_any(&item.proto, pcb_item_types::BOARD_TEXT),
            Self::BoardTextBox(item) => {
                envelope::pack_any(&item.proto, pcb_item_types::BOARD_TEXT_BOX)
            }
            Self::Field(item) => envelope::pack_any(&item.proto, pcb_item_types::FIELD),
            Self::Zone(item) => envelope::pack_any(&item.proto, pcb_item_types::ZONE),
            Self::Dimension(item) => envelope::pack_any(&item.proto, pcb_item_types::DIMENSION),
            Self::Group(item) => envelope::pack_any(&item.proto, pcb_item_types::GROUP),
            Self::ReferenceImage(item) => {
                envelope::pack_any(&item.proto, pcb_item_types::REFERENCE_IMAGE)
            }
            Self::Unknown(item) => item.raw,
        }
    }

    /// Clones and returns this item as a raw protobuf payload.
    pub fn as_any(&self) -> Any {
        self.clone().into_any()
    }

    /// Returns the item kind.
    pub fn kind(&self) -> EditablePcbItemKind {
        match self {
            Self::Track(_) => EditablePcbItemKind::Track,
            Self::Arc(_) => EditablePcbItemKind::Arc,
            Self::Via(_) => EditablePcbItemKind::Via,
            Self::Footprint(_) => EditablePcbItemKind::Footprint,
            Self::Pad(_) => EditablePcbItemKind::Pad,
            Self::BoardGraphicShape(_) => EditablePcbItemKind::BoardGraphicShape,
            Self::BoardText(_) => EditablePcbItemKind::BoardText,
            Self::BoardTextBox(_) => EditablePcbItemKind::BoardTextBox,
            Self::Field(_) => EditablePcbItemKind::Field,
            Self::Zone(_) => EditablePcbItemKind::Zone,
            Self::Dimension(_) => EditablePcbItemKind::Dimension,
            Self::Group(_) => EditablePcbItemKind::Group,
            Self::ReferenceImage(_) => EditablePcbItemKind::ReferenceImage,
            Self::Unknown(_) => EditablePcbItemKind::Unknown,
        }
    }

    /// Returns the KIID-based item id when the underlying proto has one.
    ///
    /// For `Field`, this intentionally returns `None` because fields use `FieldId`
    /// instead of a KIID and should not be exposed as fake KIID references.
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Track(item) => item.id(),
            Self::Arc(item) => item.id(),
            Self::Via(item) => item.id(),
            Self::Footprint(item) => item.id(),
            Self::Pad(item) => item.id(),
            Self::BoardGraphicShape(item) => item.id(),
            Self::BoardText(item) => item.id(),
            Self::BoardTextBox(item) => item.id(),
            Self::Field(_) => None,
            Self::Zone(item) => item.id(),
            Self::Dimension(item) => item.id(),
            Self::Group(item) => item.id(),
            Self::ReferenceImage(_) => None,
            Self::Unknown(_) => None,
        }
    }

    /// Returns layer-set semantics for this item.
    pub fn layer_set(&self) -> LayerSet {
        match self {
            Self::Track(item) => LayerSet::Single(item.layer_id()),
            Self::Arc(item) => LayerSet::Single(item.layer_id()),
            Self::Via(_) => LayerSet::Padstack,
            Self::Footprint(item) => LayerSet::Single(item.layer_id()),
            Self::Pad(_) => LayerSet::Padstack,
            Self::BoardGraphicShape(item) => LayerSet::Single(item.layer_id()),
            Self::BoardText(item) => LayerSet::Single(item.layer_id()),
            Self::BoardTextBox(item) => LayerSet::Single(item.layer_id()),
            Self::Field(item) => item
                .text()
                .map(|text| LayerSet::Single(text.layer))
                .unwrap_or(LayerSet::None),
            Self::Zone(item) => LayerSet::Multi(item.proto.layers.clone()),
            Self::Dimension(item) => LayerSet::Single(item.layer_id()),
            Self::Group(_) => LayerSet::None,
            Self::ReferenceImage(_) => LayerSet::None,
            Self::Unknown(_) => LayerSet::None,
        }
    }

    /// Attempts to set the layer id for single-layer items.
    pub fn set_layer_id(&mut self, layer_id: i32) -> Result<(), KiCadError> {
        match self {
            Self::Track(item) => {
                item.set_layer_id(layer_id);
                Ok(())
            }
            Self::Arc(item) => {
                item.set_layer_id(layer_id);
                Ok(())
            }
            Self::Footprint(item) => {
                item.set_layer_id(layer_id);
                Ok(())
            }
            Self::BoardGraphicShape(item) => {
                item.set_layer_id(layer_id);
                Ok(())
            }
            Self::BoardText(item) => {
                item.set_layer_id(layer_id);
                Ok(())
            }
            Self::BoardTextBox(item) => {
                item.set_layer_id(layer_id);
                Ok(())
            }
            Self::Field(item) => item.set_layer_id(layer_id),
            Self::Dimension(item) => {
                item.set_layer_id(layer_id);
                Ok(())
            }
            Self::Via(_) => unsupported_set_layer("via"),
            Self::Pad(_) => unsupported_set_layer("pad"),
            Self::Zone(_) => unsupported_set_layer("zone"),
            Self::Group(_) => unsupported_set_layer("group"),
            Self::ReferenceImage(_) => unsupported_set_layer("reference_image"),
            Self::Unknown(_) => unsupported_set_layer("unknown"),
        }
    }

    /// Attempts to replace layer ids for multi-layer items.
    pub fn set_layer_ids(&mut self, layer_ids: Vec<i32>) -> Result<(), KiCadError> {
        match self {
            Self::Zone(item) => {
                item.set_layer_ids(layer_ids);
                Ok(())
            }
            _ => Err(KiCadError::InvalidResponse {
                reason: format!("set_layer_ids is not supported for {:?} items", self.kind()),
            }),
        }
    }
}

impl From<EditablePcbItem> for Any {
    fn from(value: EditablePcbItem) -> Self {
        value.into_any()
    }
}

fn decode_item<T: Message + Default>(raw: &Any, type_name: &str) -> Result<T, KiCadError> {
    T::decode(raw.value.as_slice()).map_err(|err| {
        KiCadError::ProtobufDecode(format!(
            "failed decoding `{}` from `{}`: {}",
            type_name, raw.type_url, err
        ))
    })
}

fn vector2_to_proto(value: Vector2Nm) -> common_types::Vector2 {
    common_types::Vector2 {
        x_nm: value.x_nm,
        y_nm: value.y_nm,
    }
}

fn vector2_from_proto(value: common_types::Vector2) -> Vector2Nm {
    Vector2Nm {
        x_nm: value.x_nm,
        y_nm: value.y_nm,
    }
}

fn kiid_value(kiid: &Option<common_types::Kiid>) -> Option<&str> {
    kiid.as_ref().map(|id| id.value.as_str())
}

fn unsupported_set_layer(item_kind: &str) -> Result<(), KiCadError> {
    Err(KiCadError::InvalidResponse {
        reason: format!("set_layer_id is not supported for {item_kind} items"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack<T: Message>(message: &T, type_name: &str) -> Any {
        envelope::pack_any(message, type_name)
    }

    #[test]
    fn from_any_decodes_each_supported_type_url() {
        let supported = vec![
            pack(&board_types::Track::default(), pcb_item_types::TRACK),
            pack(&board_types::Arc::default(), pcb_item_types::ARC),
            pack(&board_types::Via::default(), pcb_item_types::VIA),
            pack(
                &board_types::FootprintInstance::default(),
                pcb_item_types::FOOTPRINT_INSTANCE,
            ),
            pack(&board_types::Pad::default(), pcb_item_types::PAD),
            pack(
                &board_types::BoardGraphicShape::default(),
                pcb_item_types::BOARD_GRAPHIC_SHAPE,
            ),
            pack(
                &board_types::BoardText::default(),
                pcb_item_types::BOARD_TEXT,
            ),
            pack(
                &board_types::BoardTextBox::default(),
                pcb_item_types::BOARD_TEXT_BOX,
            ),
            pack(&board_types::Field::default(), pcb_item_types::FIELD),
            pack(&board_types::Zone::default(), pcb_item_types::ZONE),
            pack(
                &board_types::Dimension::default(),
                pcb_item_types::DIMENSION,
            ),
            pack(&board_types::Group::default(), pcb_item_types::GROUP),
            pack(
                &board_types::ReferenceImage::default(),
                pcb_item_types::REFERENCE_IMAGE,
            ),
        ];

        for raw in supported {
            let item = EditablePcbItem::from_any(raw).expect("supported type should decode");
            assert_ne!(item.kind(), EditablePcbItemKind::Unknown);
        }
    }

    #[test]
    fn from_any_rejects_corrupt_known_payload() {
        let raw = Any {
            type_url: envelope::type_url(pcb_item_types::TRACK),
            value: vec![0xff, 0xff, 0xff],
        };

        let err = EditablePcbItem::from_any(raw).expect_err("corrupt known item should fail");
        assert!(err.to_string().contains("failed decoding"));
    }

    #[test]
    fn into_any_preserves_type_url_and_modified_data() {
        let mut item = EditablePcbItem::from_any(pack(
            &board_types::Track {
                layer: 3,
                ..Default::default()
            },
            pcb_item_types::TRACK,
        ))
        .expect("track should decode");

        item.set_layer_id(11).expect("track layer set should work");
        if let EditablePcbItem::Track(track) = &mut item {
            track.set_start_nm(Vector2Nm { x_nm: 5, y_nm: 7 });
        }

        let raw = item.into_any();
        assert_eq!(raw.type_url, envelope::type_url(pcb_item_types::TRACK));

        let decoded = board_types::Track::decode(raw.value.as_slice()).expect("decode track");
        assert_eq!(decoded.layer, 11);
        assert_eq!(decoded.start.map(|p| (p.x_nm, p.y_nm)), Some((5, 7)));
    }

    #[test]
    fn id_works_for_track_group_and_zone() {
        let track = EditablePcbItem::from_any(pack(
            &board_types::Track {
                id: Some(common_types::Kiid {
                    value: "track-1".to_string(),
                }),
                ..Default::default()
            },
            pcb_item_types::TRACK,
        ))
        .expect("track should decode");

        let group = EditablePcbItem::from_any(pack(
            &board_types::Group {
                id: Some(common_types::Kiid {
                    value: "group-1".to_string(),
                }),
                ..Default::default()
            },
            pcb_item_types::GROUP,
        ))
        .expect("group should decode");

        let zone = EditablePcbItem::from_any(pack(
            &board_types::Zone {
                id: Some(common_types::Kiid {
                    value: "zone-1".to_string(),
                }),
                ..Default::default()
            },
            pcb_item_types::ZONE,
        ))
        .expect("zone should decode");

        assert_eq!(track.id(), Some("track-1"));
        assert_eq!(group.id(), Some("group-1"));
        assert_eq!(zone.id(), Some("zone-1"));
    }

    #[test]
    fn layer_set_for_track_zone_via_pad_group() {
        let track = EditablePcbItem::from_any(pack(
            &board_types::Track {
                layer: 4,
                ..Default::default()
            },
            pcb_item_types::TRACK,
        ))
        .expect("track should decode");
        assert_eq!(track.layer_set(), LayerSet::Single(4));

        let zone = EditablePcbItem::from_any(pack(
            &board_types::Zone {
                layers: vec![1, 2, 3],
                ..Default::default()
            },
            pcb_item_types::ZONE,
        ))
        .expect("zone should decode");
        assert_eq!(zone.layer_set(), LayerSet::Multi(vec![1, 2, 3]));

        let via =
            EditablePcbItem::from_any(pack(&board_types::Via::default(), pcb_item_types::VIA))
                .expect("via should decode");
        assert_eq!(via.layer_set(), LayerSet::Padstack);

        let pad =
            EditablePcbItem::from_any(pack(&board_types::Pad::default(), pcb_item_types::PAD))
                .expect("pad should decode");
        assert_eq!(pad.layer_set(), LayerSet::Padstack);

        let group =
            EditablePcbItem::from_any(pack(&board_types::Group::default(), pcb_item_types::GROUP))
                .expect("group should decode");
        assert_eq!(group.layer_set(), LayerSet::None);
    }

    #[test]
    fn set_layer_id_success_for_track_text_and_error_for_via_group() {
        let mut track =
            EditablePcbItem::from_any(pack(&board_types::Track::default(), pcb_item_types::TRACK))
                .expect("track should decode");
        track.set_layer_id(9).expect("track layer set should work");
        assert_eq!(track.layer_set(), LayerSet::Single(9));

        let mut text = EditablePcbItem::from_any(pack(
            &board_types::BoardText::default(),
            pcb_item_types::BOARD_TEXT,
        ))
        .expect("text should decode");
        text.set_layer_id(40).expect("text layer set should work");
        assert_eq!(text.layer_set(), LayerSet::Single(40));

        let mut via =
            EditablePcbItem::from_any(pack(&board_types::Via::default(), pcb_item_types::VIA))
                .expect("via should decode");
        let via_err = via.set_layer_id(10).expect_err("via layer set should fail");
        assert!(via_err
            .to_string()
            .contains("set_layer_id is not supported for via items"));

        let mut group =
            EditablePcbItem::from_any(pack(&board_types::Group::default(), pcb_item_types::GROUP))
                .expect("group should decode");
        let group_err = group
            .set_layer_id(10)
            .expect_err("group layer set should fail");
        assert!(group_err
            .to_string()
            .contains("set_layer_id is not supported for group items"));
    }

    #[test]
    fn set_layer_ids_updates_zone_and_rejects_single_layer_item() {
        let mut zone = EditablePcbItem::from_any(pack(
            &board_types::Zone {
                layers: vec![1],
                ..Default::default()
            },
            pcb_item_types::ZONE,
        ))
        .expect("zone should decode");

        zone.set_layer_ids(vec![2, 3])
            .expect("zone layers should update");
        assert_eq!(zone.layer_set(), LayerSet::Multi(vec![2, 3]));

        let mut track =
            EditablePcbItem::from_any(pack(&board_types::Track::default(), pcb_item_types::TRACK))
                .expect("track should decode");
        let err = track
            .set_layer_ids(vec![2, 3])
            .expect_err("track layer list should fail");
        assert!(err.to_string().contains("set_layer_ids is not supported"));
    }

    #[test]
    fn group_item_new_any_has_correct_type_url_and_member_ids() {
        let mut group =
            GroupItem::new("my-group", vec!["item-a".to_string(), "item-b".to_string()]);
        assert_eq!(group.name(), "my-group");
        group.set_name("renamed");
        group.set_member_ids(vec!["item-c".to_string()]);

        let any = EditablePcbItem::Group(group).into_any();
        assert_eq!(any.type_url, envelope::type_url(pcb_item_types::GROUP));

        let decoded = board_types::Group::decode(any.value.as_slice()).expect("decode group");
        assert_eq!(decoded.name, "renamed");
        let members: Vec<String> = decoded.items.into_iter().map(|id| id.value).collect();
        assert_eq!(members, vec!["item-c".to_string()]);
    }

    #[test]
    fn unknown_any_round_trips_losslessly() {
        let raw = Any {
            type_url: "type.googleapis.com/kiapi.board.types.CustomThing".to_string(),
            value: vec![9, 8, 7, 6],
        };

        let editable = EditablePcbItem::from_any(raw.clone()).expect("unknown should decode");
        assert_eq!(editable.kind(), EditablePcbItemKind::Unknown);

        let roundtrip = editable.into_any();
        assert_eq!(roundtrip, raw);
    }
}
