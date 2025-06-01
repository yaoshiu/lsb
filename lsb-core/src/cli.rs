use std::path::PathBuf;

pub use clap::{Parser, Subcommand};
use clap_complete::Shell;
use lsb_core::hash::Hash;

#[derive(Parser)]
#[command(version, about, long_about)]
/// The command-line interface for the LSB (Least Significant Bit) embedding tool.
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// The number of least significant bits to use for embedding.
    #[arg(short, long, default_value = "1")]
    pub lsbs: usize,

    /// The seed for the random number generator.
    #[arg(short, long, default_value = "42")]
    pub seed: u64,
}

#[derive(Subcommand)]
pub enum Commands {
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

    /// Generate shell completions for the CLI.
    Completion {
        /// The shell to generate completions for.
        shell: Shell,
    },
}
