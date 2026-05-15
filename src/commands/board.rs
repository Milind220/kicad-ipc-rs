#[derive(Debug, Default)]
pub struct BoardCommands;

use crate::commands::base::pack_command;
use crate::proto::kiapi::board::{commands as board_commands, jobs as board_jobs};

pub const GET_BOARD_DESIGN_RULES: &str = "kiapi.board.commands.GetBoardDesignRules";
pub const SET_BOARD_DESIGN_RULES: &str = "kiapi.board.commands.SetBoardDesignRules";
pub const BOARD_DESIGN_RULES_RESPONSE: &str = "kiapi.board.commands.BoardDesignRulesResponse";
pub const GET_CUSTOM_DESIGN_RULES: &str = "kiapi.board.commands.GetCustomDesignRules";
pub const SET_CUSTOM_DESIGN_RULES: &str = "kiapi.board.commands.SetCustomDesignRules";
pub const CUSTOM_RULES_RESPONSE: &str = "kiapi.board.commands.CustomRulesResponse";

pub const RUN_BOARD_JOB_EXPORT_3D: &str = "kiapi.board.jobs.RunBoardJobExport3D";
pub const RUN_BOARD_JOB_EXPORT_RENDER: &str = "kiapi.board.jobs.RunBoardJobExportRender";
pub const RUN_BOARD_JOB_EXPORT_SVG: &str = "kiapi.board.jobs.RunBoardJobExportSvg";
pub const RUN_BOARD_JOB_EXPORT_DXF: &str = "kiapi.board.jobs.RunBoardJobExportDxf";
pub const RUN_BOARD_JOB_EXPORT_PDF: &str = "kiapi.board.jobs.RunBoardJobExportPdf";
pub const RUN_BOARD_JOB_EXPORT_PS: &str = "kiapi.board.jobs.RunBoardJobExportPs";
pub const RUN_BOARD_JOB_EXPORT_GERBERS: &str = "kiapi.board.jobs.RunBoardJobExportGerbers";
pub const RUN_BOARD_JOB_EXPORT_DRILL: &str = "kiapi.board.jobs.RunBoardJobExportDrill";
pub const RUN_BOARD_JOB_EXPORT_POSITION: &str = "kiapi.board.jobs.RunBoardJobExportPosition";
pub const RUN_BOARD_JOB_EXPORT_GENCAD: &str = "kiapi.board.jobs.RunBoardJobExportGencad";
pub const RUN_BOARD_JOB_EXPORT_IPC2581: &str = "kiapi.board.jobs.RunBoardJobExportIpc2581";
pub const RUN_BOARD_JOB_EXPORT_IPCD356: &str = "kiapi.board.jobs.RunBoardJobExportIpcD356";
pub const RUN_BOARD_JOB_EXPORT_ODB: &str = "kiapi.board.jobs.RunBoardJobExportODB";
pub const RUN_BOARD_JOB_EXPORT_STATS: &str = "kiapi.board.jobs.RunBoardJobExportStats";

pub fn get_board_design_rules(command: board_commands::GetBoardDesignRules) -> prost_types::Any {
    pack_command(&command, GET_BOARD_DESIGN_RULES)
}

pub fn set_board_design_rules(command: board_commands::SetBoardDesignRules) -> prost_types::Any {
    pack_command(&command, SET_BOARD_DESIGN_RULES)
}

pub fn get_custom_design_rules(command: board_commands::GetCustomDesignRules) -> prost_types::Any {
    pack_command(&command, GET_CUSTOM_DESIGN_RULES)
}

pub fn set_custom_design_rules(command: board_commands::SetCustomDesignRules) -> prost_types::Any {
    pack_command(&command, SET_CUSTOM_DESIGN_RULES)
}

pub fn run_board_job_export_3d(command: board_jobs::RunBoardJobExport3D) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_3D)
}

pub fn run_board_job_export_render(
    command: board_jobs::RunBoardJobExportRender,
) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_RENDER)
}

pub fn run_board_job_export_svg(command: board_jobs::RunBoardJobExportSvg) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_SVG)
}

pub fn run_board_job_export_dxf(command: board_jobs::RunBoardJobExportDxf) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_DXF)
}

pub fn run_board_job_export_pdf(command: board_jobs::RunBoardJobExportPdf) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_PDF)
}

pub fn run_board_job_export_ps(command: board_jobs::RunBoardJobExportPs) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_PS)
}

pub fn run_board_job_export_gerbers(
    command: board_jobs::RunBoardJobExportGerbers,
) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_GERBERS)
}

pub fn run_board_job_export_drill(command: board_jobs::RunBoardJobExportDrill) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_DRILL)
}

pub fn run_board_job_export_position(
    command: board_jobs::RunBoardJobExportPosition,
) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_POSITION)
}

pub fn run_board_job_export_gencad(
    command: board_jobs::RunBoardJobExportGencad,
) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_GENCAD)
}

pub fn run_board_job_export_ipc2581(
    command: board_jobs::RunBoardJobExportIpc2581,
) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_IPC2581)
}

pub fn run_board_job_export_ipcd356(
    command: board_jobs::RunBoardJobExportIpcD356,
) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_IPCD356)
}

pub fn run_board_job_export_odb(command: board_jobs::RunBoardJobExportOdb) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_ODB)
}

pub fn run_board_job_export_stats(command: board_jobs::RunBoardJobExportStats) -> prost_types::Any {
    pack_command(&command, RUN_BOARD_JOB_EXPORT_STATS)
}
