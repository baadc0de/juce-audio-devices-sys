use cmake;

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let cmake_profile = "Release";

    let mut dst = cmake::Config::new(".").profile(cmake_profile).build();


    #[cfg(target_os = "macos")]
    {
        dst.push("build");
        println!("cargo:rustc-link-search=native={}", dst.display());
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
        println!("cargo:rustc-link-lib=framework=AudioUnit");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=CoreMIDI");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }
    #[cfg(target_os = "windows")]
    {
        dst.push("build");
        dst.push(cmake_profile);

        println!("cargo:rustc-link-search=native={}", dst.display());
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=comctl32");
        println!("cargo:rustc-link-lib=shell32");
    }

    println!("cargo:rustc-link-lib=static=JuceRustBindings");

    // TODO: windows support pending
}
