use lsb_core::{embed, extract, hash::Hash};
use std::{error::Error, fs, path::PathBuf};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about)]
/// The command-line interface for the LSB (Least Significant Bit) embedding tool.
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// The number of least significant bits to use for embedding.
    #[arg(short, long, default_value = "1")]
    lsbs: usize,

    /// The seed for the random number generator.
    #[arg(short, long, default_value = "42")]
    seed: u64,
}

#[derive(Subcommand)]
enum Commands {
    /// Embed a file into a container image.
    Embed {
        /// The input file to embed.
        input: PathBuf,
        /// The container image file.
        container: PathBuf,

        /// The hashing algorithm to use.
        #[arg(long, default_value = "blake3")]
        hash: Hash,
        /// The output file for the embedded image.
        #[arg(short, long, default_value = "embedded.png")]
        output: String,
    },

    /// Extract a file from a container image.
    Extract {
        /// The container image file.
        container: PathBuf,

        /// The output file for the extracted data.
        #[arg(short, long, default_value = "extracted")]
        output: PathBuf,
    },
}

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
    }

    Ok(())
}
