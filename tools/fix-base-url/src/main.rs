use itertools::Itertools;
use std::{env, error::Error, fs};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn Error>> {
    for base in env::args().skip(1) {
        let base = fs::canonicalize(base)?;

        let mut walker = WalkDir::new(&base).into_iter().filter(|e| {
            e.as_ref()
                .map(|e| {
                    e.file_type().is_file()
                        && e.file_name()
                            .to_str()
                            .map(|p| p.ends_with(".html"))
                            .unwrap_or(false)
                })
                .unwrap_or(true)
        });

        while let Some(entry) = walker.next().transpose()? {
            let relative = entry.path().strip_prefix(&base)?;
            let mut fixed = relative.components().skip(1).map(|_| "..").join("/");
            if fixed.is_empty() {
                fixed.push('.');
            }
            fixed.push('/');
            let content = fs::read_to_string(entry.path())?;
            let content = content.replace("https://blag.nemo157.com/", &fixed);
            fs::write(entry.path(), content)?;
        }
    }

    Ok(())
}
