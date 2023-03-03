// SPDX-FileCopyrightText: 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

use regex::Regex;
use walkdir::WalkDir;

// Call with `substrate/frame` as the only argument.
fn main() {
	let folder = std::env::args().nth(1).unwrap();
	let re = Regex::new(r"^(\s+)#\[pallet::weight\(").expect("Regex is known good");
	let mut modified_files = 0;
	let mut modified_calls = 0;

	for f in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
		if f.metadata().unwrap().is_file() {
			// Only process Rust files:
			if !f.path().to_str().unwrap().ends_with(".rs") {
				continue
			}
			// Exclude the pallet-ui tests:
			if f.path().to_str().unwrap().contains("pallet_ui") {
				continue
			}

			let content = std::fs::read_to_string(f.path()).unwrap();
			let mut new_lines = Vec::with_capacity(content.lines().count());
			let mut call_index = 0;

			for (i, line) in content.lines().enumerate() {
				let m = re.captures(line);
				if let Some(m) = m {
					// Skip if there is already a call index before or after:
					if i > 0 && content.lines().nth(i - 1).unwrap().contains("pallet::call_index") {
						continue
					}
					if i + 1 < content.lines().count() &&
						content.lines().nth(i + 1).unwrap().contains("pallet::call_index")
					{
						continue
					}

					println!("{}:{} index {}", f.path().display(), i, call_index);
					new_lines.push(format!(
						"{}#[pallet::call_index({})]",
						m.get(1).unwrap().as_str(),
						call_index
					));
					call_index += 1;
				}
				new_lines.push(line.to_string());
			}

			if call_index > 0 {
				std::fs::write(f.path(), new_lines.join("\n")).unwrap();
				println!("Inserted {} indices for {}", call_index, f.path().display());
				modified_files += 1;
				modified_calls += call_index;
			}
		}
	}
	println!("Modified {} files and {} calls in total", modified_files, modified_calls);
}
