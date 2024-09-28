use color_eyre::eyre::bail;
use color_eyre::Result;
use std::process::Command;
pub fn check_for_exe(path: &str) -> Result<()> {
    let which = Command::new("which")
        .arg(path)
        .output()
        .expect("executing `which`");
    let status = which.status;
    if !status.success() {
        bail!("{path}: Executable not found");
    }
    Ok(())
}
