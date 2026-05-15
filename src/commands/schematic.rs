#[derive(Debug, Default)]
pub struct SchematicCommands;

use crate::commands::base::pack_command;
use crate::proto::kiapi::schematic::{jobs as schematic_jobs, types as schematic_types};

pub const GET_SCHEMATIC_HIERARCHY: &str = "kiapi.schematic.types.GetSchematicHierarchy";
pub const SCHEMATIC_HIERARCHY_RESPONSE: &str = "kiapi.schematic.types.SchematicHierarchyResponse";
pub const GET_SCHEMATIC_NETLIST: &str = "kiapi.schematic.types.GetSchematicNetlist";
pub const SCHEMATIC_NETLIST_RESPONSE: &str = "kiapi.schematic.types.SchematicNetlistResponse";

pub const RUN_SCHEMATIC_JOB_EXPORT_SVG: &str = "kiapi.schematic.jobs.RunSchematicJobExportSvg";
pub const RUN_SCHEMATIC_JOB_EXPORT_DXF: &str = "kiapi.schematic.jobs.RunSchematicJobExportDxf";
pub const RUN_SCHEMATIC_JOB_EXPORT_PDF: &str = "kiapi.schematic.jobs.RunSchematicJobExportPdf";
pub const RUN_SCHEMATIC_JOB_EXPORT_PS: &str = "kiapi.schematic.jobs.RunSchematicJobExportPs";
pub const RUN_SCHEMATIC_JOB_EXPORT_NETLIST: &str =
    "kiapi.schematic.jobs.RunSchematicJobExportNetlist";
pub const RUN_SCHEMATIC_JOB_EXPORT_BOM: &str = "kiapi.schematic.jobs.RunSchematicJobExportBOM";

pub fn get_schematic_hierarchy(
    command: schematic_types::GetSchematicHierarchy,
) -> prost_types::Any {
    pack_command(&command, GET_SCHEMATIC_HIERARCHY)
}

pub fn get_schematic_netlist(command: schematic_types::GetSchematicNetlist) -> prost_types::Any {
    pack_command(&command, GET_SCHEMATIC_NETLIST)
}

pub fn run_schematic_job_export_svg(
    command: schematic_jobs::RunSchematicJobExportSvg,
) -> prost_types::Any {
    pack_command(&command, RUN_SCHEMATIC_JOB_EXPORT_SVG)
}

pub fn run_schematic_job_export_dxf(
    command: schematic_jobs::RunSchematicJobExportDxf,
) -> prost_types::Any {
    pack_command(&command, RUN_SCHEMATIC_JOB_EXPORT_DXF)
}

pub fn run_schematic_job_export_pdf(
    command: schematic_jobs::RunSchematicJobExportPdf,
) -> prost_types::Any {
    pack_command(&command, RUN_SCHEMATIC_JOB_EXPORT_PDF)
}

pub fn run_schematic_job_export_ps(
    command: schematic_jobs::RunSchematicJobExportPs,
) -> prost_types::Any {
    pack_command(&command, RUN_SCHEMATIC_JOB_EXPORT_PS)
}

pub fn run_schematic_job_export_netlist(
    command: schematic_jobs::RunSchematicJobExportNetlist,
) -> prost_types::Any {
    pack_command(&command, RUN_SCHEMATIC_JOB_EXPORT_NETLIST)
}

pub fn run_schematic_job_export_bom(
    command: schematic_jobs::RunSchematicJobExportBom,
) -> prost_types::Any {
    pack_command(&command, RUN_SCHEMATIC_JOB_EXPORT_BOM)
}
