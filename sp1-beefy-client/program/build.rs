use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Load checkpoint from contracts/test/data/checkpoint.json (single shared file)
    let path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../contracts/test/data/checkpoint.json");
    let json = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "checkpoint.json not found at {}: {e}",
            path.display()
        )
    });

    let (latest_mmr_root, latest_beefy_block, current_validator_set, next_validator_set) =
        parse_checkpoint(&json);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("checkpoint.rs");
    fs::write(
        &dest,
        format!(
            r#"
static INITIAL_STATE: State = State {{
    latest_mmr_root: {latest_mmr_root},
    latest_beefy_block: {latest_beefy_block},
    current_validator_set: ValidatorSet {{
        id: {current_id},
        length: {current_length},
        root: {current_root},
    }},
    next_validator_set: ValidatorSet {{
        id: {next_id},
        length: {next_length},
        root: {next_root},
    }},
}};
"#,
            latest_mmr_root = format_array_32(&latest_mmr_root),
            latest_beefy_block = latest_beefy_block,
            current_id = current_validator_set.0,
            current_length = current_validator_set.1,
            current_root = format_array_32(&current_validator_set.2),
            next_id = next_validator_set.0,
            next_length = next_validator_set.1,
            next_root = format_array_32(&next_validator_set.2),
        ),
    )
    .expect("Failed to write checkpoint.rs");
}

fn format_array_32(arr: &[u8; 32]) -> String {
    let parts: Vec<String> = arr.iter().map(|b| format!("{b}u8")).collect();
    format!("[{}]", parts.join(", "))
}

type ValidatorSetData = (u128, u128, [u8; 32]);

fn parse_checkpoint(json: &str) -> ([u8; 32], u64, ValidatorSetData, ValidatorSetData) {
    let v: serde_json::Value = serde_json::from_str(json).expect("Invalid checkpoint JSON");
    let latest_mmr_root = parse_hex_32(
        v.get("latestMMRRoot")
            .or_else(|| v.get("latest_mmr_root"))
            .and_then(|v| v.as_str())
            .expect("missing latestMMRRoot"),
    );
    let latest_beefy_block = v
        .get("latestBeefyBlock")
        .or_else(|| v.get("latest_beefy_block"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let current = parse_validator_set(
        v.get("currentValidatorSet")
            .or_else(|| v.get("current_validator_set"))
            .expect("missing currentValidatorSet"),
    );
    let next = parse_validator_set(
        v.get("nextValidatorSet")
            .or_else(|| v.get("next_validator_set"))
            .expect("missing nextValidatorSet"),
    );
    (latest_mmr_root, latest_beefy_block, current, next)
}

fn parse_validator_set(v: &serde_json::Value) -> ValidatorSetData {
    let id = v["id"].as_u64().expect("validator set id") as u128;
    let length = v["length"].as_u64().expect("validator set length") as u128;
    let root = parse_hex_32(v["root"].as_str().expect("validator set root"));
    (id, length, root)
}

fn parse_hex_32(s: &str) -> [u8; 32] {
    let s = s.trim_start_matches("0x");
    let bytes = hex::decode(s).expect("invalid hex");
    if bytes.len() != 32 {
        panic!("expected 32 bytes, got {}", bytes.len());
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    arr
}

