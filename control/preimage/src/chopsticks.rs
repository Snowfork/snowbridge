use codec::Encode;
use handlebars::Handlebars;
use serde::Serialize;
use sp_crypto_hashing::blake2_256;
use std::io::prelude::*;
use std::{fs::File, path::PathBuf};

#[derive(Clone, Serialize, Debug)]
struct TemplateData {
    preimage: Preimage,
}

impl TemplateData {
    fn new(preimage: &[u8]) -> Self {
        TemplateData {
            preimage: preimage.into(),
        }
    }
}

#[derive(Clone, Serialize, Debug)]
struct Preimage {
    hash: String,
    bytes: String,
    size: String,
}

impl From<&[u8]> for Preimage {
    fn from(data: &[u8]) -> Self {
        Preimage {
            hash: as_hex_literal(&blake2_256(data)),
            size: format!("{}", data.len()),
            bytes: as_hex_literal(&data.to_owned().encode()),
        }
    }
}

fn as_hex_literal(s: &[u8]) -> String {
    format!("0x{}", hex::encode(s))
}

pub fn generate_chopsticks_script(
    preimage: &[u8],
    output_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = Handlebars::new();

    // Disable HTML escaping
    registry.register_escape_fn(|s| -> String { s.to_string() });

    let template = include_str!("../templates/chopsticks-execute-upgrade.js.hbs");
    let data = TemplateData::new(preimage);
    let output = registry.render_template(template, &data)?;
    let mut file = File::create(output_path)?;
    file.write_all(output.as_bytes())?;

    Ok(())
}
