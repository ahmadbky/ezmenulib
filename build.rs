use rustc_version::{version_meta, Channel};

fn main() {
    // Provides `nightly` cfg to enable `doc(cfg(...))` feature.
    if let Ok(Channel::Nightly) = version_meta().map(|v| v.channel) {
        println!("cargo:rustc-cfg=nightly");
    }
}
