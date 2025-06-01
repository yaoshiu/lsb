mod cli;

use clap::CommandFactory;
use clap_complete::generate;
use cli::*;
use lsb_core::{embed, extract};
use std::{error::Error, fs, io, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Embed {
            container,
            input,
            output,
            hash,
        } => {
            let output = PathBuf::from(output);
            let format = image::ImageFormat::from_path(&output)?;
            let ext = input.extension().and_then(|s| s.to_str()).unwrap_or("bin");

            let container =
                fs::read(container).map_err(|e| format!("Failed to read container: {}", e))?;
            let input = fs::read(&input).map_err(|e| format!("Failed to read input: {}", e))?;

            let embedded = embed(&input, ext, &container, cli.lsbs, hash, cli.seed, format)?;

            fs::write(&output, embedded).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        Commands::Extract { container, output } => {
            let container =
                fs::read(container).map_err(|e| format!("Failed to read container: {}", e))?;

            let (data, ext) = extract(&container, cli.lsbs, cli.seed)?;

            let output = output.with_extension(ext);
            fs::write(&output, data).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();

            generate(shell, &mut cmd, bin_name, &mut io::stdout());
        }
    }

    Ok(())
}
