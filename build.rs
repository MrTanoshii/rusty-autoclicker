use winresource::WindowsResource;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = WindowsResource::new();
        res.set_icon("assets/icon-256.ico").set_language(0x0409); // English (US)
        res.compile().unwrap();
    }
}
