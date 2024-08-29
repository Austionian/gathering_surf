use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use anyhow::anyhow;

/// Function to update the specified versions in 'base.html'
/// to bust caches with new code.
fn main() -> anyhow::Result<()> {
    let mut new = String::new();

    for line in fs::read_to_string("templates/base.html")?.lines() {
        if line.contains("?version=") {
            let (_, end) = line
                .split_once("version=")
                .ok_or(anyhow!("no version found in line!"))?;
            let current_version = end
                .split_once("\"")
                .ok_or(anyhow!("no qutation mark found with version"))?
                .0
                .parse::<u32>()?;
            let new_version = current_version.wrapping_add(1);

            let line = line.replace(
                &format!("version={current_version}"),
                &format!("version={new_version}"),
            );
            new.push_str(&line);
        } else {
            new.push_str(line);
        }
        new.push('\n');
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("templates/base.html")?;

    file.write_all(new.as_bytes())?;

    Ok(())
}
