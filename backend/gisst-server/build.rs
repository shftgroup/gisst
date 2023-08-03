use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const FRONT_DIR: &str = "../../frontend";
    if std::fs::metadata(FRONT_DIR)?.is_dir() {
        dbg!(Command::new("npm").arg("i").current_dir(FRONT_DIR))
            .output()
            .expect("Failed to execute npm install");
        dbg!(Command::new("npm")
            .args(["run", "build", "--workspaces", "--if-present"])
            .current_dir(FRONT_DIR));
        dbg!(Command::new("npm")
            .args(["run", "dist", "--workspaces", "--if-present"])
            .current_dir(FRONT_DIR))
        .output()
        .expect("Failed to execute npm build");
    }
    println!("cargo:rerun-if-changed={FRONT_DIR}/ra-util/src");
    println!("cargo:rerun-if-changed={FRONT_DIR}/embedv86/src");
    println!("cargo:rerun-if-changed={FRONT_DIR}/gisst-player-ui/src");
    println!("cargo:rerun-if-changed={FRONT_DIR}/frontend-web/src");
    Ok(())
}
