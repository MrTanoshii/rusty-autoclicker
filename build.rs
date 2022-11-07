use std::io;
use winres::WindowsResource;

// Issues with winres to keep in mind:
// https://github.com/mxre/winres/issues/32
// https://github.com/mxre/winres/issues/40

fn main() -> io::Result<()> {
    if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap() == "windows" {
        let mut res = WindowsResource::new();
        let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
        match env.as_str() {
            "gnu" => {
                // Not sure whether this works the same if build on windows; needs testing
                // Perfectly fine if cross compiling from linux
                res.set_ar_path("x86_64-w64-mingw32-ar")
                    .set_windres_path("x86_64-w64-mingw32-windres");
            }
            "msvc" => {}
            _ => panic!("unsupported target-env: {}", env),
        };
        res.set_icon("assets/icon-256.ico");
        res.compile()?;
    }
    Ok(())
}
