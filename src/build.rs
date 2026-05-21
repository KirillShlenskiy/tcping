#[cfg(target_os = "windows")]
fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");

    if std::env::var("PROFILE").is_ok_and(|p| p == "release") {
        winres::WindowsResource::new()
            .compile()
            .expect("failed to compile Windows resources");
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {}
