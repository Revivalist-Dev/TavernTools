//!  Actions that don't fit other modules.

use std::path::Path;

use anyhow::Result;
use base64::prelude::*;
use textwrap::{fill, Options};

use crate::tavern_card_v2::{TavernCardV2, TEXT_KEY_PNG};
use crate::tavern_card_v3::TavernCardV3;
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

/// Prints the JSON of the tavern card from path
pub fn print_json_from_path(path: &Path) -> Result<()> {
    let image = tools::read_image_from_file(path)?;
    let tag = tools::read_text_chunk(&image, TEXT_KEY_PNG)?;
    let tag = tag.map(|x| BASE64_STANDARD.decode(x).unwrap_or_default());
    let text = tag.map(|x| String::from_utf8_lossy(&x).to_string());
    let mut text = text.unwrap_or_else(|| "NO TEXT".to_string());

    // Attempt to pretty print the JSON. If it fails, just print the raw text.
    let pretty_text = pretty_json(&text).unwrap_or_else(|_| text.clone());

    let options = Options::new(textwrap::termwidth());
    let filled_text = fill(&pretty_text, options);
    println!("{}", filled_text);
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
