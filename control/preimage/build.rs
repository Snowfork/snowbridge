fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script (i.e. recompile)
    println!("cargo:rerun-if-changed=polkadot-metadata.bin");
    println!("cargo:rerun-if-changed=bridge-hub-metadata.bin");
}
