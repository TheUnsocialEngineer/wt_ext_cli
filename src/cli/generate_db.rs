use clap::{Arg, ArgAction, Command, ValueHint};

pub fn generate_db() -> Command {
	Command::new("generate_db")
		.long_flag("generate_db")
		.about("Generates a tank database from .blk files")
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
                .required(false)
                .default_value("tank_db.json")
		)
}
