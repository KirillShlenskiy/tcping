#[cfg(target_os = "windows")]
extern crate winres;

#[cfg(target_os = "windows")]
fn main() {
    if std::env::var("PROFILE").unwrap() == "release" {
        let res = winres::WindowsResource::new();
        res.compile().unwrap();
    }
}

#[cfg(not(target_os = "windows"))]
fn main() { }