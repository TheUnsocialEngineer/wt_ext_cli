use clap::{Arg, ArgAction, Command, ValueHint};

pub fn extract_presets_from_blk() -> Command {
	Command::new("extract_presets_from_blk")
		.long_flag("extract_presets_from_blk")
		.about("Extracts presets from .blk files")
		.arg(
			Arg::new("input_folder")
				.short('i')
				.long("input_dir")
				.help("Folder with .blk files inside")
				.required(true)
				.value_hint(ValueHint::AnyPath)
		)
		.arg(
			Arg::new("output_file")
				.short('o')
				.long("output_file")
				.help("Target File that will be created to contain output JSON")
				.value_hint(ValueHint::FilePath)
		)
}
