use std::env;
use std::fs;
use std::path::Path;
use hex::FromHex;


use std::fmt;
use std::error::Error;

/// Wraps `&'static str` and implements the `Error` trait for it.
#[derive(Debug)]
struct StringError {
    error: &'static str
}

impl StringError {
    fn new(error: &'static str) -> Self {
        return StringError { error }
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        self.error
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.error)
    }
}

const DEFAULT_APP_ID: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let out_dir = env::var("OUT_DIR").unwrap();
    let appid_envs: Vec<&str> = vec!["ETH_APP_ID", "ERC20_APP_ID"];

    for appid_env in appid_envs.into_iter() {
        let app_id: Vec<u8> = match env::var(appid_env).ok() {
            Some(value) => {
                let stripped = match value.strip_prefix("0x") {
                    Some(rest) => rest,
                    None => return Err(Box::new(StringError::new("Invalid Ethereum address")))
                };
                let app_id: Vec<u8> = stripped.from_hex().map_err(|_| Box::new(StringError::new("Cannot decode address")))?;
                if app_id.len() != 20 {
                    return Err(Box::new(StringError::new("Invalid Ethereum address")));
                }
                app_id
            }
            None => DEFAULT_APP_ID.clone().to_vec()
        };
        let lowercased = appid_env.to_owned().to_ascii_lowercase();
        let mut path = Path::new(&out_dir).join(lowercased);
        path.set_extension("rs");
        fs::write(&path, format!("{:?}", app_id.as_slice()))?;
    }

    println!("cargo:rerun-if-env-changed=ETH_APP_ID");
    println!("cargo:rerun-if-env-changed=ERC20_APP_ID");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
