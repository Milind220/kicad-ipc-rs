#[derive(Debug, Default)]
pub struct BaseCommands;

pub const PROTOBUF_EMPTY_RESPONSE: &str = "google.protobuf.Empty";
pub const RUN_JOB_RESPONSE: &str = "kiapi.common.types.RunJobResponse";

pub fn type_url(type_name: &str) -> String {
    format!("type.googleapis.com/{type_name}")
}

pub fn pack_command<T: prost::Message>(command: &T, type_name: &str) -> prost_types::Any {
    prost_types::Any {
        type_url: type_url(type_name),
        value: command.encode_to_vec(),
    }
}
