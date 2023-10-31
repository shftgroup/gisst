use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const FRONT_DIR: &str = "../../frontend";
    if std::fs::metadata(FRONT_DIR)
        .map(|md| md.is_dir())
        .unwrap_or(false)
    {
        dbg!(Command::new("npm").arg("i").current_dir(FRONT_DIR)).output()?;
        dbg!(Command::new("npm")
            .args(["run", "build", "--workspaces", "--if-present"])
            .current_dir(FRONT_DIR))
        .output()?;
        dbg!(Command::new("npm")
            .args(["run", "dist", "--workspaces", "--if-present"])
            .current_dir(FRONT_DIR))
        .output()?;
    }

    Command::new("mkdir").args(["-p", "../web-dist/assets/backend"]).output()?;
    Command::new("cp").args(["-r", "src/assets/*", "../web-dist/assets/backend"]).output()?;

    println!("cargo:rerun-if-changed={FRONT_DIR}/ra-util/src");
    println!("cargo:rerun-if-changed={FRONT_DIR}/embedv86/src");
    println!("cargo:rerun-if-changed={FRONT_DIR}/gisst-player-ui/src");
    println!("cargo:rerun-if-changed={FRONT_DIR}/frontend-web/src");
    println!("cargo:rerun-if-changed={FRONT_DIR}/frontend-web/public");
    Ok(())
}
