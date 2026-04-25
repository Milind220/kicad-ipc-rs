use super::*;
use crate::proto::kiapi::{board::types as board_types, common::types as common_types};
use prost::Message;
use prost_types::Any;

fn pack<T: Message>(message: &T, type_name: &str) -> Any {
    envelope::pack_any(message, type_name)
}

#[test]
fn from_any_decodes_each_supported_type_url() {
    let supported = vec![
        pack(&board_types::Track::default(), pcb_item_type_urls::TRACK),
        pack(&board_types::Arc::default(), pcb_item_type_urls::ARC),
        pack(&board_types::Via::default(), pcb_item_type_urls::VIA),
        pack(
            &board_types::FootprintInstance::default(),
            pcb_item_type_urls::FOOTPRINT_INSTANCE,
        ),
        pack(&board_types::Pad::default(), pcb_item_type_urls::PAD),
        pack(
            &board_types::BoardGraphicShape::default(),
            pcb_item_type_urls::BOARD_GRAPHIC_SHAPE,
        ),
        pack(
            &board_types::BoardText::default(),
            pcb_item_type_urls::BOARD_TEXT,
        ),
        pack(
            &board_types::BoardTextBox::default(),
            pcb_item_type_urls::BOARD_TEXT_BOX,
        ),
        pack(&board_types::Field::default(), pcb_item_type_urls::FIELD),
        pack(&board_types::Zone::default(), pcb_item_type_urls::ZONE),
        pack(
            &board_types::Dimension::default(),
            pcb_item_type_urls::DIMENSION,
        ),
        pack(&board_types::Group::default(), pcb_item_type_urls::GROUP),
        pack(
            &board_types::ReferenceImage::default(),
            pcb_item_type_urls::REFERENCE_IMAGE,
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
        type_url: envelope::type_url(pcb_item_type_urls::TRACK),
        value: vec![0xff, 0xff, 0xff],
    };

    let err = EditablePcbItem::from_any(raw).expect_err("corrupt known item should fail");
    assert!(err.to_string().contains("failed decoding"));
}

#[test]
fn try_from_any_matches_from_any() {
    let raw = pack(&board_types::Track::default(), pcb_item_type_urls::TRACK);
    let item = EditablePcbItem::try_from(raw).expect("track should decode");
    assert_eq!(item.kind(), EditablePcbItemKind::Track);
}

#[test]
fn into_any_preserves_type_url_and_modified_data() {
    let mut item = EditablePcbItem::from_any(pack(
        &board_types::Track {
            layer: 3,
            ..Default::default()
        },
        pcb_item_type_urls::TRACK,
    ))
    .expect("track should decode");

    item.set_layer_id(11).expect("track layer set should work");
    if let EditablePcbItem::Track(track) = &mut item {
        track.set_start_nm(Vector2Nm { x_nm: 5, y_nm: 7 });
    }

    let raw = item.into_any();
    assert_eq!(raw.type_url, envelope::type_url(pcb_item_type_urls::TRACK));

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
        pcb_item_type_urls::TRACK,
    ))
    .expect("track should decode");

    let group = EditablePcbItem::from_any(pack(
        &board_types::Group {
            id: Some(common_types::Kiid {
                value: "group-1".to_string(),
            }),
            ..Default::default()
        },
        pcb_item_type_urls::GROUP,
    ))
    .expect("group should decode");

    let zone = EditablePcbItem::from_any(pack(
        &board_types::Zone {
            id: Some(common_types::Kiid {
                value: "zone-1".to_string(),
            }),
            ..Default::default()
        },
        pcb_item_type_urls::ZONE,
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
        pcb_item_type_urls::TRACK,
    ))
    .expect("track should decode");
    assert_eq!(track.layer_set(), LayerSet::Single(4));

    let zone = EditablePcbItem::from_any(pack(
        &board_types::Zone {
            layers: vec![1, 2, 3],
            ..Default::default()
        },
        pcb_item_type_urls::ZONE,
    ))
    .expect("zone should decode");
    assert_eq!(zone.layer_set(), LayerSet::Multi(vec![1, 2, 3]));

    let via =
        EditablePcbItem::from_any(pack(&board_types::Via::default(), pcb_item_type_urls::VIA))
            .expect("via should decode");
    assert_eq!(via.layer_set(), LayerSet::Padstack);

    let pad =
        EditablePcbItem::from_any(pack(&board_types::Pad::default(), pcb_item_type_urls::PAD))
            .expect("pad should decode");
    assert_eq!(pad.layer_set(), LayerSet::Padstack);

    let group = EditablePcbItem::from_any(pack(
        &board_types::Group::default(),
        pcb_item_type_urls::GROUP,
    ))
    .expect("group should decode");
    assert_eq!(group.layer_set(), LayerSet::None);
}

#[test]
fn set_layer_id_success_for_track_text_and_error_for_via_group() {
    let mut track = EditablePcbItem::from_any(pack(
        &board_types::Track::default(),
        pcb_item_type_urls::TRACK,
    ))
    .expect("track should decode");
    track.set_layer_id(9).expect("track layer set should work");
    assert_eq!(track.layer_set(), LayerSet::Single(9));

    let mut text = EditablePcbItem::from_any(pack(
        &board_types::BoardText::default(),
        pcb_item_type_urls::BOARD_TEXT,
    ))
    .expect("text should decode");
    text.set_layer_id(40).expect("text layer set should work");
    assert_eq!(text.layer_set(), LayerSet::Single(40));

    let mut via =
        EditablePcbItem::from_any(pack(&board_types::Via::default(), pcb_item_type_urls::VIA))
            .expect("via should decode");
    let via_err = via.set_layer_id(10).expect_err("via layer set should fail");
    assert!(via_err
        .to_string()
        .contains("set_layer_id is not supported for via items"));

    let mut group = EditablePcbItem::from_any(pack(
        &board_types::Group::default(),
        pcb_item_type_urls::GROUP,
    ))
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
        pcb_item_type_urls::ZONE,
    ))
    .expect("zone should decode");

    zone.set_layer_ids(vec![2, 3])
        .expect("zone layers should update");
    assert_eq!(zone.layer_set(), LayerSet::Multi(vec![2, 3]));

    let mut track = EditablePcbItem::from_any(pack(
        &board_types::Track::default(),
        pcb_item_type_urls::TRACK,
    ))
    .expect("track should decode");
    let err = track
        .set_layer_ids(vec![2, 3])
        .expect_err("track layer list should fail");
    assert!(err.to_string().contains("set_layer_ids is not supported"));
}

#[test]
fn group_item_new_any_has_correct_type_url_and_member_ids() {
    let mut group = GroupItem::new("my-group", vec!["item-a".to_string(), "item-b".to_string()]);
    assert_eq!(group.name(), "my-group");
    group.set_name("renamed");
    group.set_member_ids(vec!["item-c".to_string()]);

    let any = EditablePcbItem::Group(group).into_any();
    assert_eq!(any.type_url, envelope::type_url(pcb_item_type_urls::GROUP));

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
