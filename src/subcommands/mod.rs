use std::str::FromStr;

use clap::ArgMatches;
use color_eyre::eyre::{bail, Context, Result};
use log::LevelFilter;

use crate::{
	logging::init_logging,
	subcommands::{
		unpack_dxp_and_grp::unpack_dxp_and_grp,
		unpack_raw_blk::unpack_raw_blk,
		unpack_vromf::unpack_vromf,
		vromf_version::vromf_version,
		extract_presets_from_blk::extract_presets_from_blk,
		extract_ammo_from_blk::extract_ammo_from_blk,
		generate_db::build_tank_db,
	},
	COMMIT_HASH,
};

mod unpack_dxp_and_grp;
mod unpack_raw_blk;
pub mod unpack_vromf;
mod vromf_version;
pub mod extract_presets_from_blk;
pub mod extract_ammo_from_blk;
pub mod generate_db;

pub fn branch_subcommands(args: ArgMatches) -> Result<()> {
	let log_level = if let Some(lvl) = args.get_one::<String>("log_level") {
		LevelFilter::from_str(lvl).context(format!("Incorrect log-level provided, expected one of [Trace, Debug, Info, Warn, Error], found {lvl}"))?
	} else {
		LevelFilter::Warn
	};
	init_logging(log_level)?;

	match args.subcommand() {
		Some(("unpack_raw_blk", args)) => {
			unpack_raw_blk(args)?;
		},
		Some(("unpack_vromf", args)) => {
			unpack_vromf(args)?;
		},
		Some(("unpack_dxp_and_grp", args)) => {
			unpack_dxp_and_grp(args)?;
		},
		Some(("extract_presets_from_blk", args)) => {
			if let (Some(input_folder), Some(output_file)) = (args.get_one::<String>("input_folder"), args.get_one::<String>("output_file")) {
				extract_presets_from_blk(input_folder, output_file)?;
			} else {
				bail!("Both input_folder and output_file arguments are required for extract_presets");
			}
		},
		Some(("extract_ammo_from_blk", args)) => {
			if let (Some(input_folder), Some(output_file)) = (args.get_one::<String>("input_folder"), args.get_one::<String>("output_file")) {
				extract_ammo_from_blk(input_folder, output_file)?;
			} else {
				bail!("Both input_folder and output_file arguments are required for extract_ammo");
			}
		},
		Some(("generate_db", args)) => {
			if let (Some(input_folder), Some(output_file)) = (args.get_one::<String>("input_folder"), args.get_one::<String>("output_file")) {
				build_tank_db(input_folder, output_file)?;
			} else {
				bail!("Both input_folder and output_file arguments are required for generate_db");
			}
		},
		Some(("get_instruction_manual", _)) => {
			open::that("https://github.com/Warthunder-Open-Source-Foundation/wt_ext_cli/blob/master/usage_manual.md").context("Attempted to show manual in browser, but something unexpected failed")?;
		},
		Some(("hash", _)) => {
			println!("https://github.com/Warthunder-Open-Source-Foundation/wt_ext_cli/commit/{COMMIT_HASH}");
		},
		Some(("vromf_version", args)) => {
			vromf_version(args)?;
		},
		_ => {
			if let Some((command, _)) = args.subcommand() {
				bail!("Unrecognized subcommand: {:}", command)
			} else {
				bail!("Missing Subcommand argument")
			}
		},
	}
	Ok(())
}
