#[cfg(target_os = "windows")]
extern crate winres;

#[cfg(target_os = "windows")]
fn main() {
    if std::env::var("PROFILE").is_ok_and(|p| p == "release") {
        winres::WindowsResource::new().compile().unwrap();
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {}
