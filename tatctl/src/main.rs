use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use libtatted::{BiLevel, ImagePreProcessor, InkyFourColorMap, Resolution};
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
        command: DisplayCommands,
    },

    /// Image pre-processing steps for e-ink rendering
    Image {
        /// The image to pre-process for rendering
        #[arg(short, long)]
        image_path: Utf8PathBuf,

        /// Out path for the pre-processed image
        #[arg(short, long, default_value_t = Utf8PathBuf::from("./output.png"))]
        out_path: Utf8PathBuf,
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Image {
            image_path,
            out_path,
        } => {
            // let preproc = ImagePreProcessor::new(BiLevel, Resolution::new(400, 300));
            // let index_image = preproc.prepare_from_path(image_path)?;
            // index_image.save(out_path)?;

            let preproc = ImagePreProcessor::new(InkyFourColorMap, Resolution::new(400, 300));
            let index_image = preproc.prepare_dither_from_path(image_path)?;
            index_image.save(out_path)?;
        }
        Commands::Display { command } => match command {
            DisplayCommands::Detect => {
                todo!()
            }
            DisplayCommands::Clear => {
                todo!()
            }
            DisplayCommands::RenderImage { path: _ } => {
                todo!()
            }
            DisplayCommands::RenderColor { color: _ } => {
                todo!()
            }
        },
    }

    Ok(())
}
