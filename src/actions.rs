//!  Actions that don't fit other modules.

use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::fs;

use anyhow::{bail, Context, Result};
use base64::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use textwrap::{fill, Options};

use crate::card_formats::tavern_card_v2::{TavernCardV2, TEXT_KEY_PNG};
use crate::card_formats::tavern_card_v3::TavernCardV3;
use crate::tools;

enum AnyTavernCard {
    V2(TavernCardV2),
    V3(TavernCardV3),
}

impl AnyTavernCard {
    fn from_png_image(image_data: &bytes::Bytes) -> Result<Self> {
        // Try V3 first
        if let Ok(card_v3) = TavernCardV3::from_png_image(image_data) {
            return Ok(AnyTavernCard::V3(card_v3));
        }
        // Fallback to V2
        if let Ok(card_v2) = TavernCardV2::from_png_image(image_data) {
            return Ok(AnyTavernCard::V2(card_v2));
        }
        bail!("Failed to parse image as either TavernCardV2 or TavernCardV3");
    }
}

impl Display for AnyTavernCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyTavernCard::V2(card) => write!(f, "{}", card),
            AnyTavernCard::V3(card) => write!(f, "{}", card),
        }
    }
}

/// Prints the content of tavern card from a given file path
pub fn print_tavern_card_from_path(path: &Path) -> Result<()> {
    let image = tools::read_image_from_file(path)?;
    let card = AnyTavernCard::from_png_image(&image)?;
    println!("{}", card);

    Ok(())
}

/// Prints the content of a JSON tavern card from a given file path
pub fn print_json_card_from_path(path: &Path) -> Result<()> {
    let json_text = std::fs::read_to_string(path)?;
    let card_v3_result = serde_json::from_str::<TavernCardV3>(&json_text);
    let card_v2_result = serde_json::from_str::<TavernCardV2>(&json_text);

    if let Ok(card_v3) = card_v3_result {
        println!("{}", card_v3);
    } else if let Ok(card_v2) = card_v2_result {
        println!("{}", card_v2);
    } else {
        bail!("Failed to parse JSON as either TavernCardV2 or TavernCardV3");
    }

    Ok(())
}

/// Prints the JSON of the tavern card from path
pub fn print_json_from_path(path: &Path) -> Result<()> {
    let image = tools::read_image_from_file(path)?;
    let tag = tools::read_text_chunk(&image, TEXT_KEY_PNG)?;
    let tag = tag.map(|x| BASE64_STANDARD.decode(x).unwrap_or_default());
    let text = tag.map(|x| String::from_utf8_lossy(&x).to_string());
    let text = text.unwrap_or_else(|| "NO TEXT".to_string());

    // Attempt to pretty print the JSON. If it fails, just print the raw text.
    let pretty_text = pretty_json(&text).unwrap_or_else(|_| text.clone());

    let options = Options::new(textwrap::termwidth());
    let filled_text = fill(&pretty_text, options);
    println!("{}", filled_text);
    Ok(())
}

/// Processes all PNG cards in the input directory.
///
/// For each card, it extracts the JSON and image data, saving them to the output directory.
/// If a card cannot be processed, it is moved to an appropriate issue subfolder.
pub fn process_all_cards(
    input_dir: &Path,
    output_dir: &Path,
    issue_dir: &Path,
) -> Result<()> {
    info!("Starting batch processing of cards from: {}", input_dir.display());

    let input_files: Vec<PathBuf> = fs::read_dir(input_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "png"))
        .collect();

    if input_files.is_empty() {
        info!("No PNG files found in the input directory: {}", input_dir.display());
        return Ok(());
    }

    let pb = ProgressBar::new(input_files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
            )?
            .progress_chars("#>-"),
    );

    for file_path in input_files {
        let file_name = file_path.file_name().context("Invalid file name")?;
        let stem = file_path.file_stem().context("Invalid file stem")?;
        pb.set_message(format!("Processing {}", file_name.to_string_lossy()));

        let output_json_path = output_dir.join(format!("{}.json", stem.to_string_lossy()));
        let output_image_path = output_dir.join(file_name);

        let result = (|| -> Result<()> {
            // Extract JSON
            extract_json_from_png(&file_path, &output_json_path)?;
            // Extract Image
            extract_image_from_png(&file_path, &output_image_path)?;
            Ok(())
        })();

        if let Err(e) = result {
            error!("Failed to process {}: {}", file_path.display(), e);
            let issue_sub_dir = if e.to_string().contains("Failed to parse") {
                issue_dir.join("format")
            } else if e.to_string().contains("No Chara entry") {
                issue_dir.join("no_data")
            } else {
                issue_dir.join("other")
            };
            fs::create_dir_all(&issue_sub_dir)?;
            let destination_path = issue_sub_dir.join(file_name);
            fs::rename(&file_path, &destination_path)?;
            pb.println(format!(
                "Moved {} to {} due to error: {}",
                file_name.to_string_lossy(),
                issue_sub_dir.display(),
                e
            ));
        } else {
            info!("Successfully processed {}", file_name.to_string_lossy());
        }
        pb.inc(1);
    }

    pb.finish_with_message("Batch processing complete!");
    Ok(())
}

/// Extracts the JSON from a PNG image and saves it to a specified JSON file.
pub fn extract_json_from_png(image_path: &Path, output_path: &Path) -> Result<()> {
    let image = tools::read_image_from_file(image_path)?;
    let tag = tools::read_text_chunk(&image, TEXT_KEY_PNG)?;
    let tag = tag.map(|x| BASE64_STANDARD.decode(x).unwrap_or_default());
    let text = tag.map(|x| String::from_utf8_lossy(&x).to_string());
    let text = text.unwrap_or_else(|| "NO TEXT".to_string());

    let pretty_text = pretty_json(&text).unwrap_or_else(|_| text.clone());
    std::fs::write(output_path, pretty_text)?;
    Ok(())
}

/// Extracts the image data from a PNG image (without the embedded JSON) and saves it to a specified PNG file.
pub fn extract_image_from_png(image_path: &Path, output_path: &Path) -> Result<()> {
    let image = tools::read_image_from_file(image_path)?;
    let image_without_text = tools::remove_text_chunk(&image, TEXT_KEY_PNG)?;
    tools::write_image_to_file(&image_without_text, output_path)?;
    Ok(())
}

fn pretty_json(text: &str) -> Result<String> {
    // A JSON deserializer. You can use any Serde Deserializer here.
    let mut deserializer = serde_json::Deserializer::from_str(text);

    // A compacted JSON serializer. You can use any Serde Serializer here.
    let mut buf: Vec<u8> = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"   ");
    let mut serializer =
        serde_json::Serializer::with_formatter(&mut buf, formatter);

    serde_transcode::transcode(&mut deserializer, &mut serializer)?;

    Ok(String::from_utf8_lossy(&buf).to_string())
}
