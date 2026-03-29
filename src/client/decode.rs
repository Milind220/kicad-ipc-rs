//! PCB item decoding from raw protobuf `Any` payloads into typed `PcbItem` variants.

use super::mappers::*;
use crate::envelope;
use crate::error::KiCadError;
use crate::model::board::*;
use crate::proto::kiapi::board::types as board_types;
use crate::proto::kiapi::common::types as common_types;
pub(crate) fn map_graphic_shape_kind(shape: Option<&common_types::GraphicShape>) -> Option<String> {
    let geometry = shape?.geometry.as_ref()?;
    Some(match geometry {
        common_types::graphic_shape::Geometry::Segment(_) => "SEGMENT".to_string(),
        common_types::graphic_shape::Geometry::Rectangle(_) => "RECTANGLE".to_string(),
        common_types::graphic_shape::Geometry::Arc(_) => "ARC".to_string(),
        common_types::graphic_shape::Geometry::Circle(_) => "CIRCLE".to_string(),
        common_types::graphic_shape::Geometry::Polygon(_) => "POLYGON".to_string(),
        common_types::graphic_shape::Geometry::Bezier(_) => "BEZIER".to_string(),
    })
}

pub(crate) fn map_dimension_style(
    style: Option<board_types::dimension::DimensionStyle>,
) -> Option<PcbDimensionStyle> {
    let style = style?;
    match style {
        board_types::dimension::DimensionStyle::Aligned(aligned) => {
            Some(PcbDimensionStyle::Aligned {
                start_nm: aligned.start.map(map_vector2_nm),
                end_nm: aligned.end.map(map_vector2_nm),
                height_nm: map_optional_distance_nm(aligned.height),
                extension_height_nm: map_optional_distance_nm(aligned.extension_height),
            })
        }
        board_types::dimension::DimensionStyle::Orthogonal(orthogonal) => {
            let alignment = common_types::AxisAlignment::try_from(orthogonal.alignment)
                .map(|value| value.as_str_name().to_string())
                .unwrap_or_else(|_| format!("UNKNOWN({})", orthogonal.alignment));

            Some(PcbDimensionStyle::Orthogonal {
                start_nm: orthogonal.start.map(map_vector2_nm),
                end_nm: orthogonal.end.map(map_vector2_nm),
                height_nm: map_optional_distance_nm(orthogonal.height),
                extension_height_nm: map_optional_distance_nm(orthogonal.extension_height),
                alignment: Some(alignment),
            })
        }
        board_types::dimension::DimensionStyle::Radial(radial) => Some(PcbDimensionStyle::Radial {
            center_nm: radial.center.map(map_vector2_nm),
            radius_point_nm: radial.radius_point.map(map_vector2_nm),
            leader_length_nm: map_optional_distance_nm(radial.leader_length),
        }),
        board_types::dimension::DimensionStyle::Leader(leader) => {
            let border_style = board_types::DimensionTextBorderStyle::try_from(leader.border_style)
                .map(|value| value.as_str_name().to_string())
                .unwrap_or_else(|_| format!("UNKNOWN({})", leader.border_style));
            Some(PcbDimensionStyle::Leader {
                start_nm: leader.start.map(map_vector2_nm),
                end_nm: leader.end.map(map_vector2_nm),
                border_style: Some(border_style),
            })
        }
        board_types::dimension::DimensionStyle::Center(center) => Some(PcbDimensionStyle::Center {
            center_nm: center.center.map(map_vector2_nm),
            end_nm: center.end.map(map_vector2_nm),
        }),
    }
}

pub(crate) fn map_pad_type(value: i32) -> PcbPadType {
    match board_types::PadType::try_from(value) {
        Ok(board_types::PadType::PtPth) => PcbPadType::Pth,
        Ok(board_types::PadType::PtSmd) => PcbPadType::Smd,
        Ok(board_types::PadType::PtEdgeConnector) => PcbPadType::EdgeConnector,
        Ok(board_types::PadType::PtNpth) => PcbPadType::Npth,
        _ => PcbPadType::Unknown(value),
    }
}

pub(crate) fn map_zone_type(value: i32) -> PcbZoneType {
    match board_types::ZoneType::try_from(value) {
        Ok(board_types::ZoneType::ZtCopper) => PcbZoneType::Copper,
        Ok(board_types::ZoneType::ZtGraphical) => PcbZoneType::Graphical,
        Ok(board_types::ZoneType::ZtRuleArea) => PcbZoneType::RuleArea,
        Ok(board_types::ZoneType::ZtTeardrop) => PcbZoneType::Teardrop,
        _ => PcbZoneType::Unknown(value),
    }
}

pub(crate) fn decode_pcb_items(items: Vec<prost_types::Any>) -> Result<Vec<PcbItem>, KiCadError> {
    items.into_iter().map(decode_pcb_item).collect()
}

pub(crate) fn decode_pcb_item(item: prost_types::Any) -> Result<PcbItem, KiCadError> {
    if item.type_url == envelope::type_url("kiapi.board.types.Track") {
        let track = decode_any::<board_types::Track>(&item, "kiapi.board.types.Track")?;
        return Ok(PcbItem::Track(PcbTrack {
            id: track.id.map(|id| id.value),
            start_nm: track.start.map(map_vector2_nm),
            end_nm: track.end.map(map_vector2_nm),
            width_nm: map_optional_distance_nm(track.width),
            locked: map_lock_state(track.locked),
            layer: layer_to_model(track.layer),
            net: map_optional_net(track.net),
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Arc") {
        let arc = decode_any::<board_types::Arc>(&item, "kiapi.board.types.Arc")?;
        return Ok(PcbItem::Arc(PcbArc {
            id: arc.id.map(|id| id.value),
            start_nm: arc.start.map(map_vector2_nm),
            mid_nm: arc.mid.map(map_vector2_nm),
            end_nm: arc.end.map(map_vector2_nm),
            width_nm: map_optional_distance_nm(arc.width),
            locked: map_lock_state(arc.locked),
            layer: layer_to_model(arc.layer),
            net: map_optional_net(arc.net),
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Via") {
        let via = decode_any::<board_types::Via>(&item, "kiapi.board.types.Via")?;
        return Ok(PcbItem::Via(PcbVia {
            id: via.id.map(|id| id.value),
            position_nm: via.position.map(map_vector2_nm),
            via_type: map_via_type(via.r#type),
            locked: map_lock_state(via.locked),
            layers: map_via_layers(via.pad_stack.as_ref()),
            pad_stack: map_pad_stack(via.pad_stack.as_ref()),
            net: map_optional_net(via.net),
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.FootprintInstance") {
        let footprint = decode_any::<board_types::FootprintInstance>(
            &item,
            "kiapi.board.types.FootprintInstance",
        )?;
        let reference = footprint
            .reference_field
            .as_ref()
            .and_then(|field| field.text.as_ref())
            .and_then(|board_text| board_text.text.as_ref())
            .map(|text| text.text.clone())
            .filter(|value| !value.is_empty());
        let value = footprint
            .value_field
            .as_ref()
            .and_then(|field| field.text.as_ref())
            .and_then(|board_text| board_text.text.as_ref())
            .map(|text| text.text.clone())
            .filter(|value| !value.is_empty());
        let datasheet = footprint
            .datasheet_field
            .as_ref()
            .and_then(|field| field.text.as_ref())
            .and_then(|board_text| board_text.text.as_ref())
            .map(|text| text.text.clone())
            .filter(|value| !value.is_empty());
        let description = footprint
            .description_field
            .as_ref()
            .and_then(|field| field.text.as_ref())
            .and_then(|board_text| board_text.text.as_ref())
            .map(|text| text.text.clone())
            .filter(|value| !value.is_empty());
        let definition_item_count = footprint
            .definition
            .as_ref()
            .map(|definition| definition.items.len())
            .unwrap_or(0);
        let pad_count = footprint
            .definition
            .as_ref()
            .map(|definition| {
                definition
                    .items
                    .iter()
                    .filter(|entry| entry.type_url == envelope::type_url("kiapi.board.types.Pad"))
                    .count()
            })
            .unwrap_or(0);
        let symbol_sheet_name = (!footprint.symbol_sheet_name.is_empty())
            .then_some(footprint.symbol_sheet_name.clone());
        let symbol_sheet_filename = (!footprint.symbol_sheet_filename.is_empty())
            .then_some(footprint.symbol_sheet_filename.clone());
        let symbol_footprint_filters = (!footprint.symbol_footprint_filters.is_empty())
            .then_some(footprint.symbol_footprint_filters.clone());
        let has_symbol_path = footprint.symbol_path.is_some();
        let symbol_link = if has_symbol_path
            || symbol_sheet_name.is_some()
            || symbol_sheet_filename.is_some()
            || symbol_footprint_filters.is_some()
        {
            Some(PcbFootprintSymbolLink {
                has_symbol_path,
                sheet_name: symbol_sheet_name,
                sheet_filename: symbol_sheet_filename,
                footprint_filters: symbol_footprint_filters,
            })
        } else {
            None
        };

        return Ok(PcbItem::Footprint(PcbFootprint {
            id: footprint.id.map(|id| id.value),
            reference,
            position_nm: footprint.position.map(map_vector2_nm),
            orientation_deg: footprint.orientation.map(|angle| angle.value_degrees),
            layer: layer_to_model(footprint.layer),
            locked: map_lock_state(footprint.locked),
            value,
            datasheet,
            description,
            has_attributes: footprint.attributes.is_some(),
            has_overrides: footprint.overrides.is_some(),
            has_definition: footprint.definition.is_some(),
            definition_item_count,
            symbol_link,
            pad_count,
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Pad") {
        let pad = decode_any::<board_types::Pad>(&item, "kiapi.board.types.Pad")?;
        let symbol_pin = pad.symbol_pin.map(|pin| {
            let pin_type = common_types::ElectricalPinType::try_from(pin.r#type)
                .map(|value| value.as_str_name().to_string())
                .unwrap_or_else(|_| format!("UNKNOWN({})", pin.r#type));
            PcbSymbolPinInfo {
                name: pin.name,
                pin_type: Some(pin_type),
                no_connect: pin.no_connect,
            }
        });
        return Ok(PcbItem::Pad(PcbPad {
            id: pad.id.map(|id| id.value),
            locked: map_lock_state(pad.locked),
            number: pad.number,
            pad_type: map_pad_type(pad.r#type),
            position_nm: pad.position.map(map_vector2_nm),
            pad_stack: map_pad_stack(pad.pad_stack.as_ref()),
            copper_clearance_override_nm: map_optional_distance_nm(pad.copper_clearance_override),
            pad_to_die_length_nm: map_optional_distance_nm(pad.pad_to_die_length),
            pad_to_die_delay_as: pad.pad_to_die_delay.map(|value| value.value_as),
            symbol_pin,
            net: map_optional_net(pad.net),
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.BoardGraphicShape") {
        let shape = decode_any::<board_types::BoardGraphicShape>(
            &item,
            "kiapi.board.types.BoardGraphicShape",
        )?;
        let geometry_kind = map_graphic_shape_kind(shape.shape.as_ref());
        let geometry = map_graphic_shape_geometry(shape.shape.as_ref());
        let stroke_width_nm = shape
            .shape
            .as_ref()
            .and_then(|graphic| graphic.attributes.as_ref())
            .and_then(|attrs| attrs.stroke.as_ref())
            .and_then(|stroke| stroke.width)
            .map(|width| width.value_nm);
        let stroke_style = shape
            .shape
            .as_ref()
            .and_then(|graphic| graphic.attributes.as_ref())
            .and_then(|attrs| attrs.stroke.as_ref())
            .map(|stroke| {
                common_types::StrokeLineStyle::try_from(stroke.style)
                    .map(|value| value.as_str_name().to_string())
                    .unwrap_or_else(|_| format!("UNKNOWN({})", stroke.style))
            });
        let fill_type = shape
            .shape
            .as_ref()
            .and_then(|graphic| graphic.attributes.as_ref())
            .and_then(|attrs| attrs.fill.as_ref())
            .map(|fill| {
                common_types::GraphicFillType::try_from(fill.fill_type)
                    .map(|value| value.as_str_name().to_string())
                    .unwrap_or_else(|_| format!("UNKNOWN({})", fill.fill_type))
            });
        return Ok(PcbItem::BoardGraphicShape(PcbBoardGraphicShape {
            id: shape.id.map(|id| id.value),
            layer: layer_to_model(shape.layer),
            locked: map_lock_state(shape.locked),
            net: map_optional_net(shape.net),
            geometry_kind,
            geometry,
            stroke_width_nm,
            stroke_style,
            fill_type,
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.BoardText") {
        let text = decode_any::<board_types::BoardText>(&item, "kiapi.board.types.BoardText")?;
        let (body, position_nm, hyperlink, attributes) = if let Some(value) = text.text {
            let hyperlink = (!value.hyperlink.is_empty()).then_some(value.hyperlink.clone());
            let body = (!value.text.is_empty()).then_some(value.text.clone());
            (
                body,
                value.position.map(map_vector2_nm),
                hyperlink,
                map_text_attributes(value.attributes),
            )
        } else {
            (None, None, None, None)
        };

        return Ok(PcbItem::BoardText(PcbBoardText {
            id: text.id.map(|id| id.value),
            layer: layer_to_model(text.layer),
            text: body,
            position_nm,
            hyperlink,
            attributes,
            knockout: text.knockout,
            locked: map_lock_state(text.locked),
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.BoardTextBox") {
        let textbox =
            decode_any::<board_types::BoardTextBox>(&item, "kiapi.board.types.BoardTextBox")?;
        let (body, top_left_nm, bottom_right_nm, attributes) = if let Some(value) = textbox.textbox
        {
            (
                (!value.text.is_empty()).then_some(value.text.clone()),
                value.top_left.map(map_vector2_nm),
                value.bottom_right.map(map_vector2_nm),
                map_text_attributes(value.attributes),
            )
        } else {
            (None, None, None, None)
        };
        return Ok(PcbItem::BoardTextBox(PcbBoardTextBox {
            id: textbox.id.map(|id| id.value),
            layer: layer_to_model(textbox.layer),
            text: body,
            top_left_nm,
            bottom_right_nm,
            attributes,
            locked: map_lock_state(textbox.locked),
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Field") {
        let field = decode_any::<board_types::Field>(&item, "kiapi.board.types.Field")?;
        let text = field
            .text
            .and_then(|board_text| board_text.text)
            .map(|value| value.text);
        return Ok(PcbItem::Field(PcbField {
            name: field.name,
            visible: field.visible,
            text,
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Zone") {
        let zone = decode_any::<board_types::Zone>(&item, "kiapi.board.types.Zone")?;
        let has_copper_settings = matches!(
            zone.settings,
            Some(board_types::zone::Settings::CopperSettings(_))
        );
        let has_rule_area_settings = matches!(
            zone.settings,
            Some(board_types::zone::Settings::RuleAreaSettings(_))
        );
        let border_style = zone.border.as_ref().map(|border| {
            board_types::ZoneBorderStyle::try_from(border.style)
                .map(|value| value.as_str_name().to_string())
                .unwrap_or_else(|_| format!("UNKNOWN({})", border.style))
        });
        let border_pitch_nm = zone
            .border
            .as_ref()
            .and_then(|border| map_optional_distance_nm(border.pitch));
        let layer_properties = zone
            .layer_properties
            .iter()
            .map(|entry| PcbZoneLayerProperty {
                layer: layer_to_model(entry.layer),
                hatching_offset_nm: entry.hatching_offset.map(map_vector2_nm),
            })
            .collect::<Vec<_>>();
        let layers = zone
            .layers
            .iter()
            .copied()
            .map(layer_to_model)
            .collect::<Vec<_>>();

        return Ok(PcbItem::Zone(PcbZone {
            id: zone.id.map(|id| id.value),
            name: zone.name,
            zone_type: map_zone_type(zone.r#type),
            layers,
            layer_count: zone.layers.len(),
            priority: zone.priority,
            locked: map_lock_state(zone.locked),
            filled: zone.filled,
            polygon_count: zone.filled_polygons.len(),
            outline_polygon_count: zone.outline.map_or(0, |outline| outline.polygons.len()),
            has_copper_settings,
            has_rule_area_settings,
            border_style,
            border_pitch_nm,
            layer_properties,
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Dimension") {
        let dimension = decode_any::<board_types::Dimension>(&item, "kiapi.board.types.Dimension")?;
        let style_kind = dimension.dimension_style.as_ref().map(|value| match value {
            board_types::dimension::DimensionStyle::Aligned(_) => "ALIGNED".to_string(),
            board_types::dimension::DimensionStyle::Orthogonal(_) => "ORTHOGONAL".to_string(),
            board_types::dimension::DimensionStyle::Radial(_) => "RADIAL".to_string(),
            board_types::dimension::DimensionStyle::Leader(_) => "LEADER".to_string(),
            board_types::dimension::DimensionStyle::Center(_) => "CENTER".to_string(),
        });
        let style = map_dimension_style(dimension.dimension_style);
        let override_text =
            (!dimension.override_text.is_empty()).then_some(dimension.override_text);
        let prefix = (!dimension.prefix.is_empty()).then_some(dimension.prefix);
        let suffix = (!dimension.suffix.is_empty()).then_some(dimension.suffix);
        let unit = board_types::DimensionUnit::try_from(dimension.unit)
            .map(|value| value.as_str_name().to_string())
            .unwrap_or_else(|_| format!("UNKNOWN({})", dimension.unit));
        let unit_format = board_types::DimensionUnitFormat::try_from(dimension.unit_format)
            .map(|value| value.as_str_name().to_string())
            .unwrap_or_else(|_| format!("UNKNOWN({})", dimension.unit_format));
        let arrow_direction =
            board_types::DimensionArrowDirection::try_from(dimension.arrow_direction)
                .map(|value| value.as_str_name().to_string())
                .unwrap_or_else(|_| format!("UNKNOWN({})", dimension.arrow_direction));
        let precision = board_types::DimensionPrecision::try_from(dimension.precision)
            .map(|value| value.as_str_name().to_string())
            .unwrap_or_else(|_| format!("UNKNOWN({})", dimension.precision));
        let text_position = board_types::DimensionTextPosition::try_from(dimension.text_position)
            .map(|value| value.as_str_name().to_string())
            .unwrap_or_else(|_| format!("UNKNOWN({})", dimension.text_position));

        return Ok(PcbItem::Dimension(PcbDimension {
            id: dimension.id.map(|id| id.value),
            layer: layer_to_model(dimension.layer),
            locked: map_lock_state(dimension.locked),
            text: dimension.text.map(|value| value.text),
            style_kind,
            style,
            override_text_enabled: dimension.override_text_enabled,
            override_text,
            prefix,
            suffix,
            unit: Some(unit),
            unit_format: Some(unit_format),
            arrow_direction: Some(arrow_direction),
            precision: Some(precision),
            suppress_trailing_zeroes: dimension.suppress_trailing_zeroes,
            line_thickness_nm: map_optional_distance_nm(dimension.line_thickness),
            arrow_length_nm: map_optional_distance_nm(dimension.arrow_length),
            extension_offset_nm: map_optional_distance_nm(dimension.extension_offset),
            text_position: Some(text_position),
            keep_text_aligned: dimension.keep_text_aligned,
        }));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Group") {
        let group = decode_any::<board_types::Group>(&item, "kiapi.board.types.Group")?;
        return Ok(PcbItem::Group(PcbGroup {
            id: group.id.map(|id| id.value),
            name: group.name,
            item_count: group.items.len(),
            item_ids: group.items.into_iter().map(|item| item.value).collect(),
        }));
    }

    Ok(PcbItem::Unknown(PcbUnknownItem {
        type_url: item.type_url,
        raw_len: item.value.len(),
    }))
}

pub(crate) fn pad_netlist_from_footprint_items(
    footprint_items: Vec<prost_types::Any>,
) -> Result<Vec<PadNetEntry>, KiCadError> {
    let mut entries = Vec::new();
    for item in footprint_items {
        if item.type_url != envelope::type_url("kiapi.board.types.FootprintInstance") {
            continue;
        }

        let footprint = decode_any::<board_types::FootprintInstance>(
            &item,
            "kiapi.board.types.FootprintInstance",
        )?;

        let footprint_reference = footprint
            .reference_field
            .as_ref()
            .and_then(|field| field.text.as_ref())
            .and_then(|board_text| board_text.text.as_ref())
            .map(|text| text.text.clone())
            .filter(|value| !value.is_empty());

        let footprint_id = footprint.id.as_ref().map(|id| id.value.clone());

        let footprint_definition = footprint.definition.unwrap_or_default();
        for sub_item in footprint_definition.items {
            if sub_item.type_url != envelope::type_url("kiapi.board.types.Pad") {
                continue;
            }

            let pad = decode_any::<board_types::Pad>(&sub_item, "kiapi.board.types.Pad")?;
            let (net_code, net_name) = match pad.net {
                Some(net) => {
                    let code = net.code.map(|code| code.value);
                    let name = if net.name.is_empty() {
                        None
                    } else {
                        Some(net.name)
                    };
                    (code, name)
                }
                None => (None, None),
            };

            entries.push(PadNetEntry {
                footprint_reference: footprint_reference.clone(),
                footprint_id: footprint_id.clone(),
                pad_id: pad.id.map(|id| id.value),
                pad_number: pad.number,
                net_code,
                net_name,
            });
        }
    }

    Ok(entries)
}
