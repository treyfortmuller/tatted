use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use tatctl::CliColors;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Turn debugging information on
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Display manipulation and rendering
    Display {
        #[command(subcommand)]
        cmd: DisplayCommands,
    },

    /// Image pre-processing steps for e-ink rendering
    Image {
        /// The image to pre-process for rendering
        #[arg(short, long)]
        image: Utf8PathBuf,

        /// Outpath for the pre-processed image
        #[arg(short, long, default_value_t = Utf8PathBuf::from("./output.png"))]
        outpath: Utf8PathBuf,
    },
}

/// Subcommands for display manipulation
#[derive(Clone, Debug, Subcommand)]
pub enum DisplayCommands {
    /// Detect required peripherals and read display EEPROM
    Detect,

    /// Clear the display, all white pixels
    Clear,

    /// Render an arbitrary image
    RenderImage {
        /// Filepath to the image to render
        #[arg(short, long)]
        path: Utf8PathBuf,
    },

    /// Render a solid color
    RenderColor {
        /// Which solid color to render
        #[arg(short, long, default_value_t = CliColors::Red)]
        color: CliColors,
    },
}

fn main() {
    let _cli = Cli::parse();

    println!("Hellow world!");
}
