fn main() {
	println!("cargo:rerun-if-changed=metadata-snowbase.scale");
	println!("cargo:rerun-if-changed=metadata-snowblink.scale");
}
