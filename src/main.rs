#![allow(dead_code)]

use anyhow::Result;
use clap::{Parser, ValueHint};
use std::path::{Path, PathBuf};

mod actions;
mod card_providers;
mod deasterisk;
mod card_formats;
mod tools;
//mod example;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_INPUT_PATH: &str = "inventory/input";
const DEFAULT_OUTPUT_PATH: &str = "inventory/output";
const DEFAULT_LOG_PATH: &str = "inventory/last_run.log";
const DEFAULT_ISSUE_PATH: &str = "inventory/issue";
const DEFAULT_ISSUE_PATH_FORMAT: &str = "inventory/issue/format";
const DEFAULT_ISSUE_PATH_NODATA: &str = "inventory/issue/no_data";

#[derive(Parser, Debug)]
#[command(author = "Barafu Albino <barafu_develops@albino.email",
     version = APP_VERSION,
     about = "Tools for tavern cards", long_about = None)]
#[group(multiple = false)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// If no command is provided, "print" command is used by default.
    #[arg(value_hint = ValueHint::FilePath)]
    card_path: Option<PathBuf>,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Download tavern card from BackyardAI
    #[command(name = "baya_get")]
    #[command(arg_required_else_help = true)]
    BayaGet {
        /// URL at Backyard AI website to download from
        #[arg()]
        url: String,
        /// Path to output file. Defaults to "inventory/output/<character_name>.png"
        #[arg(value_hint = ValueHint::FilePath, default_value = DEFAULT_OUTPUT_PATH)]
        output_path: PathBuf,
    },
    /// Remove paired asterisks from text in tavern card. Makes a copy of the image and renames it to de8.<old_name.png>
    #[command(arg_required_else_help = true)]
    De8 {
        /// Path to image.png. Defaults to "inventory/input/<filename.png>"
        #[arg(value_hint = ValueHint::FilePath, default_value = DEFAULT_INPUT_PATH)]
        path: PathBuf,

        /// Overwrite output file if it exists already
        #[arg(long)]
        force: bool,
    },
    /// Print the content of the card
    #[command(arg_required_else_help = true)]
    Print {
        /// Path to image.png. Defaults to "inventory/input/<filename.png>"
        #[arg(value_hint = ValueHint::FilePath, default_value = DEFAULT_INPUT_PATH)]
        path: PathBuf,
    },
    /// Print the JSON of the card
    #[command(name = "print_all")]
    #[command(arg_required_else_help = true)]
    PrintJson {
        /// Path to image.png. Defaults to "inventory/input/<filename.png>"
        #[arg(value_hint = ValueHint::FilePath, default_value = DEFAULT_INPUT_PATH)]
        path: PathBuf,
    },
    /// Print the content of a JSON card file
    #[command(name = "print_json_file")]
    #[command(arg_required_else_help = true)]
    PrintJsonFile {
        /// Path to JSON file. Defaults to "inventory/input/<filename>.json"
        #[arg(value_hint = ValueHint::FilePath, default_value = DEFAULT_INPUT_PATH)]
        path: PathBuf,
    },
    /// Extract JSON from a PNG card and save it to a .json file
    #[command(name = "extract_json")]
    #[command(arg_required_else_help = true)]
    ExtractJson {
        /// Path to the PNG image file. Defaults to "inventory/input/<filename>.png"
        #[arg(value_hint = ValueHint::FilePath, default_value = DEFAULT_INPUT_PATH)]
        image_path: PathBuf,
        /// Path to the output JSON file. Defaults to "inventory/output/<filename>.json"
        #[arg(value_hint = ValueHint::FilePath, default_value = DEFAULT_OUTPUT_PATH)]
        output_path: PathBuf,
    },
    /// Extract the image from a PNG card (without embedded JSON) and save it to a new .png file
    #[command(name = "extract_image")]
    #[command(arg_required_else_help = true)]
    ExtractImage {
        /// Path to the PNG image file. Defaults to "inventory/input/<filename>.png"
        #[arg(value_hint = ValueHint::FilePath, default_value = DEFAULT_INPUT_PATH)]
        image_path: PathBuf,
        /// Path to the output PNG file. Defaults to "inventory/output/<filename>.png"
        #[arg(value_hint = ValueHint::FilePath, default_value = DEFAULT_OUTPUT_PATH)]
        output_path: PathBuf,
    },
    /// Process all PNG cards in the input directory, extracting JSON and image, and handling errors.
    #[command(name = "process_all")]
    ProcessAll {
        /// Path to the input directory. Defaults to "inventory/input"
        #[arg(value_hint = ValueHint::DirPath, default_value = DEFAULT_INPUT_PATH)]
        input_dir: PathBuf,
        /// Path to the output directory. Defaults to "inventory/output"
        #[arg(value_hint = ValueHint::DirPath, default_value = DEFAULT_OUTPUT_PATH)]
        output_dir: PathBuf,
        /// Path to the issue directory. Defaults to "inventory/issue"
        #[arg(value_hint = ValueHint::DirPath, default_value = DEFAULT_ISSUE_PATH)]
        issue_dir: PathBuf,
    },
}

fn main() {
    // Prepare debug logging.
    #[cfg(debug_assertions)]
    {
        use env_logger::Builder;
        use std::fs::{self, File};
        let inventory_dir = Path::new("inventory");
        if !inventory_dir.exists() {
            fs::create_dir_all(inventory_dir).expect("Can't create inventory directory");
        }
        let input_dir = Path::new(DEFAULT_INPUT_PATH);
        if !input_dir.exists() {
            fs::create_dir_all(input_dir).expect("Can't create input directory");
        }
        let output_dir = Path::new(DEFAULT_OUTPUT_PATH);
        if !output_dir.exists() {
            fs::create_dir_all(output_dir).expect("Can't create output directory");
        }
        let issue_dir = Path::new(DEFAULT_ISSUE_PATH);
        if !issue_dir.exists() {
            fs::create_dir_all(issue_dir).expect("Can't create issue directory");
        }
        let issue_format_dir = Path::new(DEFAULT_ISSUE_PATH_FORMAT);
        if !issue_format_dir.exists() {
            fs::create_dir_all(issue_format_dir).expect("Can't create issue/format directory");
        }
        let issue_nodata_dir = Path::new(DEFAULT_ISSUE_PATH_NODATA);
        if !issue_nodata_dir.exists() {
            fs::create_dir_all(issue_nodata_dir).expect("Can't create issue/no_data directory");
        }

        let target = Box::new(
            File::create(DEFAULT_LOG_PATH).expect("Can't create file"),
        );

        Builder::new()
            .target(env_logger::Target::Pipe(target))
            .filter(None, log::LevelFilter::Info)
            .init();
    }

    // Print intro
    println!("tavern card tools v{}", APP_VERSION);

    if let Err(err) = parse_args() {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}

fn parse_args() -> Result<()> {
    let args = Cli::parse();

    if args.card_path.is_none() && args.command.is_none() {
        eprintln!("Error: No command given");
        // println!("{}", Cli::);
        std::process::exit(1);
    }

    if let Some(card_path) = args.card_path {
        actions::print_tavern_card_from_path(&card_path)?;
        return Ok(());
    }

    match args.command.unwrap() {
        Commands::BayaGet { url, output_path } => {
            card_providers::baya_download::download_card_from_baya_url(&url, &output_path)?
        }
        Commands::De8 { path, force } => {
            deasterisk::deasterisk_tavern_file(&path, force)?
        }
        Commands::Print { path } => {
            actions::print_tavern_card_from_path(&path)?
        }
        Commands::PrintJson { path } => actions::print_json_from_path(&path)?,
        Commands::PrintJsonFile { path } => actions::print_json_card_from_path(&path)?,
        Commands::ExtractJson {
            image_path,
            output_path,
        } => actions::extract_json_from_png(&image_path, &output_path)?,
        Commands::ExtractImage {
            image_path,
            output_path,
        } => actions::extract_image_from_png(&image_path, &output_path)?,
        Commands::ProcessAll {
            input_dir,
            output_dir,
            issue_dir,
        } => actions::process_all_cards(&input_dir, &output_dir, &issue_dir)?,
    };
    Ok(())
}
