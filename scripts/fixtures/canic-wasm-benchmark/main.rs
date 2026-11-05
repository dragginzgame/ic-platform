//! Diagnostic driver copied into a captured Canic Host examples directory.
//! Uses its public build API and caller-selected admitted tools.

use canic_contracts::ids::ReleaseBuildId;
use canic_host::{
    build_toolchain::BuildToolchain,
    canister_build::{
        CanisterArtifactBuilder, CanisterBuildProfile, WorkspaceBuildContext, copy_icp_wasm_output,
    },
    icp_config::resolve_icp_build_network_from_root,
};
use std::path::PathBuf;

#[derive(clap::Parser)]
struct Arguments {
    canister_name: String,
    profile: CanisterBuildProfile,
    workspace_root: PathBuf,
    icp_root: PathBuf,
    config_path: PathBuf,
    #[arg(long)]
    environment: String,
    #[arg(long)]
    release_build_id: ReleaseBuildId,
    #[arg(long)]
    ic_wasm: PathBuf,
    #[arg(long)]
    wasm_opt: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Arguments as clap::Parser>::parse();
    let workspace_root = args.workspace_root.canonicalize()?;
    let icp_root = args.icp_root.canonicalize()?;
    let config_path = workspace_root.join(args.config_path).canonicalize()?;
    let build_network = resolve_icp_build_network_from_root(&icp_root, &args.environment)?;
    let toolchain = BuildToolchain::resolve_with_paths(
        args.profile,
        Some(&args.ic_wasm),
        Some(&args.wasm_opt),
    )?;
    let context = WorkspaceBuildContext {
        role: args.canister_name.clone(),
        profile: args.profile,
        environment: args.environment,
        build_network,
        config_path,
        workspace_root,
        icp_root,
        local_replica: None,
        refresh_canonical_infrastructure_did: false,
        release_build_id: Some(args.release_build_id),
    };
    eprintln!("{}", context.lines().join("\n"));
    let output = CanisterArtifactBuilder::with_toolchain(toolchain)
        .build_workspace_canister_artifact(&context)?;
    copy_icp_wasm_output(&args.canister_name, &output)?;
    println!("{}", output.wasm_gz_path.display());
    Ok(())
}
