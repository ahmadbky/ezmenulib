use rustc_version::{version_meta, Channel};

fn main() {
    if let Ok(Channel::Nightly) = version_meta().map(|v| v.channel) {
        println!("cargo:rustc-cfg=nightly");
    }
}
