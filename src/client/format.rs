use super::mappers::*;
use crate::envelope;
use crate::error::KiCadError;
use crate::model::board::*;
use crate::model::common::*;
use crate::proto::kiapi::board::types as board_types;
use crate::proto::kiapi::common::types as common_types;
use prost::Message;
pub(crate) fn selection_item_detail(item: &prost_types::Any) -> Result<String, KiCadError> {
    if item.type_url == envelope::type_url("kiapi.board.types.Track") {
        let track = decode_any::<board_types::Track>(item, "kiapi.board.types.Track")?;
        return Ok(format_track_selection_detail(track));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Arc") {
        let arc = decode_any::<board_types::Arc>(item, "kiapi.board.types.Arc")?;
        return Ok(format_arc_selection_detail(arc));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Via") {
        let via = decode_any::<board_types::Via>(item, "kiapi.board.types.Via")?;
        return Ok(format_via_selection_detail(via));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.FootprintInstance") {
        let footprint = decode_any::<board_types::FootprintInstance>(
            item,
            "kiapi.board.types.FootprintInstance",
        )?;
        return Ok(format_footprint_selection_detail(footprint));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Field") {
        let field = decode_any::<board_types::Field>(item, "kiapi.board.types.Field")?;
        return Ok(format_field_selection_detail(field));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.BoardText") {
        let text = decode_any::<board_types::BoardText>(item, "kiapi.board.types.BoardText")?;
        return Ok(format_board_text_selection_detail(text));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.BoardTextBox") {
        let textbox =
            decode_any::<board_types::BoardTextBox>(item, "kiapi.board.types.BoardTextBox")?;
        return Ok(format_board_textbox_selection_detail(textbox));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Pad") {
        let pad = decode_any::<board_types::Pad>(item, "kiapi.board.types.Pad")?;
        return Ok(format_pad_selection_detail(pad));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.BoardGraphicShape") {
        let shape = decode_any::<board_types::BoardGraphicShape>(
            item,
            "kiapi.board.types.BoardGraphicShape",
        )?;
        return Ok(format_board_graphic_shape_selection_detail(shape));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Zone") {
        let zone = decode_any::<board_types::Zone>(item, "kiapi.board.types.Zone")?;
        return Ok(format_zone_selection_detail(zone));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Dimension") {
        let dimension = decode_any::<board_types::Dimension>(item, "kiapi.board.types.Dimension")?;
        return Ok(format_dimension_selection_detail(dimension));
    }

    if item.type_url == envelope::type_url("kiapi.board.types.Group") {
        let group = decode_any::<board_types::Group>(item, "kiapi.board.types.Group")?;
        return Ok(format_group_selection_detail(group));
    }

    Ok(format!("unparsed payload ({} bytes)", item.value.len()))
}

pub(crate) fn format_track_selection_detail(track: board_types::Track) -> String {
    let id = track.id.map_or_else(|| "-".to_string(), |id| id.value);
    let start = track
        .start
        .map_or_else(|| "-".to_string(), |v| format!("{},{}", v.x_nm, v.y_nm));
    let end = track
        .end
        .map_or_else(|| "-".to_string(), |v| format!("{},{}", v.x_nm, v.y_nm));
    let width = track
        .width
        .map_or_else(|| "-".to_string(), |w| w.value_nm.to_string());
    let layer = layer_to_model(track.layer).name;
    let net = track
        .net
        .map(|n| format!("{}:{}", n.code.map_or(0, |c| c.value), n.name))
        .unwrap_or_else(|| "-".to_string());
    format!("track id={id} start_nm={start} end_nm={end} width_nm={width} layer={layer} net={net}")
}

pub(crate) fn format_arc_selection_detail(arc: board_types::Arc) -> String {
    let id = arc.id.map_or_else(|| "-".to_string(), |id| id.value);
    let start = arc
        .start
        .map_or_else(|| "-".to_string(), |v| format!("{},{}", v.x_nm, v.y_nm));
    let mid = arc
        .mid
        .map_or_else(|| "-".to_string(), |v| format!("{},{}", v.x_nm, v.y_nm));
    let end = arc
        .end
        .map_or_else(|| "-".to_string(), |v| format!("{},{}", v.x_nm, v.y_nm));
    let width = arc
        .width
        .map_or_else(|| "-".to_string(), |w| w.value_nm.to_string());
    let layer = layer_to_model(arc.layer).name;
    let net = arc
        .net
        .map(|n| format!("{}:{}", n.code.map_or(0, |c| c.value), n.name))
        .unwrap_or_else(|| "-".to_string());
    format!(
        "arc id={id} start_nm={start} mid_nm={mid} end_nm={end} width_nm={width} layer={layer} net={net}"
    )
}

pub(crate) fn format_via_selection_detail(via: board_types::Via) -> String {
    let id = via.id.map_or_else(|| "-".to_string(), |id| id.value);
    let position = via
        .position
        .map_or_else(|| "-".to_string(), |v| format!("{},{}", v.x_nm, v.y_nm));
    let net = via
        .net
        .map(|n| format!("{}:{}", n.code.map_or(0, |c| c.value), n.name))
        .unwrap_or_else(|| "-".to_string());
    let via_type = board_types::ViaType::try_from(via.r#type)
        .map(|value| value.as_str_name().to_string())
        .unwrap_or_else(|_| format!("UNKNOWN({})", via.r#type));
    let layers = map_via_layers(via.pad_stack.as_ref());
    let pad_layers = layers
        .as_ref()
        .map(|row| format_layer_names(&row.padstack_layers))
        .unwrap_or_else(|| "-".to_string());
    let drill_start = layers
        .as_ref()
        .and_then(|row| row.drill_start_layer.as_ref())
        .map(|layer| layer.name.as_str())
        .unwrap_or("-");
    let drill_end = layers
        .as_ref()
        .and_then(|row| row.drill_end_layer.as_ref())
        .map(|layer| layer.name.as_str())
        .unwrap_or("-");

    format!(
        "via id={id} pos_nm={position} type={via_type} net={net} pad_layers={pad_layers} drill_span={drill_start}->{drill_end}"
    )
}

pub(crate) fn format_layer_names(layers: &[BoardLayerInfo]) -> String {
    if layers.is_empty() {
        return "-".to_string();
    }

    layers
        .iter()
        .map(|layer| layer.name.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn format_footprint_selection_detail(
    footprint: board_types::FootprintInstance,
) -> String {
    let id = footprint.id.map_or_else(|| "-".to_string(), |id| id.value);
    let reference = footprint
        .reference_field
        .as_ref()
        .and_then(|field| field.text.as_ref())
        .and_then(|board_text| board_text.text.as_ref())
        .map(|text| text.text.clone())
        .unwrap_or_else(|| "-".to_string());
    let position = footprint
        .position
        .map_or_else(|| "-".to_string(), |v| format!("{},{}", v.x_nm, v.y_nm));
    let orientation_deg = footprint.orientation.map_or_else(
        || "-".to_string(),
        |orientation| orientation.value_degrees.to_string(),
    );
    let layer = layer_to_model(footprint.layer).name;
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
    format!(
        "footprint id={id} ref={reference} pos_nm={position} orientation_deg={orientation_deg} layer={layer} pad_count={pad_count}"
    )
}

pub(crate) fn format_field_selection_detail(field: board_types::Field) -> String {
    let text = field
        .text
        .as_ref()
        .and_then(|board_text| board_text.text.as_ref())
        .map(|text| text.text.clone())
        .unwrap_or_else(|| "-".to_string());
    format!(
        "field name={} visible={} text={}",
        field.name, field.visible, text
    )
}

pub(crate) fn format_board_text_selection_detail(text: board_types::BoardText) -> String {
    let id = text.id.map_or_else(|| "-".to_string(), |id| id.value);
    let layer = layer_to_model(text.layer).name;
    let body = text
        .text
        .as_ref()
        .map(|value| value.text.clone())
        .unwrap_or_else(|| "-".to_string());
    format!("text id={id} layer={layer} text={body}")
}

pub(crate) fn format_board_textbox_selection_detail(textbox: board_types::BoardTextBox) -> String {
    let id = textbox.id.map_or_else(|| "-".to_string(), |id| id.value);
    let layer = layer_to_model(textbox.layer).name;
    let body = textbox
        .textbox
        .as_ref()
        .map(|value| value.text.clone())
        .unwrap_or_else(|| "-".to_string());
    format!("textbox id={id} layer={layer} text={body}")
}

pub(crate) fn format_pad_selection_detail(pad: board_types::Pad) -> String {
    let id = pad.id.map_or_else(|| "-".to_string(), |id| id.value);
    let pad_type = board_types::PadType::try_from(pad.r#type)
        .map(|value| value.as_str_name().to_string())
        .unwrap_or_else(|_| format!("UNKNOWN({})", pad.r#type));
    let position = pad
        .position
        .map_or_else(|| "-".to_string(), |v| format!("{},{}", v.x_nm, v.y_nm));
    let net = pad
        .net
        .map(|n| format!("{}:{}", n.code.map_or(0, |c| c.value), n.name))
        .unwrap_or_else(|| "-".to_string());
    format!(
        "pad id={id} number={} type={pad_type} pos_nm={position} net={net}",
        pad.number
    )
}

pub(crate) fn format_board_graphic_shape_selection_detail(
    shape: board_types::BoardGraphicShape,
) -> String {
    let id = shape.id.map_or_else(|| "-".to_string(), |id| id.value);
    let layer = layer_to_model(shape.layer).name;
    let net = shape
        .net
        .map(|n| format!("{}:{}", n.code.map_or(0, |c| c.value), n.name))
        .unwrap_or_else(|| "-".to_string());
    let geometry = shape
        .shape
        .as_ref()
        .map(|graphic| format!("{:?}", graphic.geometry))
        .unwrap_or_else(|| "-".to_string());
    format!("graphic id={id} layer={layer} net={net} geometry={geometry}")
}

pub(crate) fn format_zone_selection_detail(zone: board_types::Zone) -> String {
    let id = zone.id.map_or_else(|| "-".to_string(), |id| id.value);
    let zone_type = board_types::ZoneType::try_from(zone.r#type)
        .map(|value| value.as_str_name().to_string())
        .unwrap_or_else(|_| format!("UNKNOWN({})", zone.r#type));
    format!(
        "zone id={id} name={} type={} layer_count={} filled={} polygon_count={}",
        zone.name,
        zone_type,
        zone.layers.len(),
        zone.filled,
        zone.filled_polygons.len()
    )
}

pub(crate) fn format_dimension_selection_detail(dimension: board_types::Dimension) -> String {
    let id = dimension.id.map_or_else(|| "-".to_string(), |id| id.value);
    let layer = layer_to_model(dimension.layer).name;
    let text = dimension
        .text
        .as_ref()
        .map(|value| value.text.clone())
        .unwrap_or_else(|| "-".to_string());
    let style = format!("{:?}", dimension.dimension_style);
    format!(
        "dimension id={id} layer={layer} text={} style={style}",
        text
    )
}

pub(crate) fn format_group_selection_detail(group: board_types::Group) -> String {
    let id = group.id.map_or_else(|| "-".to_string(), |id| id.value);
    format!(
        "group id={id} name={} item_count={}",
        group.name,
        group.items.len()
    )
}

pub(crate) fn any_to_pretty_debug(item: &prost_types::Any) -> Result<String, KiCadError> {
    macro_rules! debug_any {
        ($(($url:literal, $ty:ty)),* $(,)?) => {
            $(
                if item.type_url == envelope::type_url($url) {
                    let value = decode_any::<$ty>(item, $url)?;
                    return Ok(format!("{:#?}", value));
                }
            )*
        };
    }

    debug_any!(
        ("kiapi.board.types.Track", board_types::Track),
        ("kiapi.board.types.Arc", board_types::Arc),
        ("kiapi.board.types.Via", board_types::Via),
        (
            "kiapi.board.types.FootprintInstance",
            board_types::FootprintInstance
        ),
        ("kiapi.board.types.Pad", board_types::Pad),
        (
            "kiapi.board.types.BoardGraphicShape",
            board_types::BoardGraphicShape
        ),
        ("kiapi.board.types.BoardText", board_types::BoardText),
        ("kiapi.board.types.BoardTextBox", board_types::BoardTextBox),
        ("kiapi.board.types.Field", board_types::Field),
        ("kiapi.board.types.Zone", board_types::Zone),
        ("kiapi.board.types.Dimension", board_types::Dimension),
        ("kiapi.board.types.Group", board_types::Group),
    );

    Ok(format!(
        "unparsed_any type_url={} raw_len={}",
        item.type_url,
        item.value.len()
    ))
}
