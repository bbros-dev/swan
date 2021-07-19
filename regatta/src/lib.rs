#[macro_use]
extern crate clap;
extern crate clap_verbosity_flag;

use anyhow::{anyhow, bail, Context, Result};
use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use std::ffi::OsString;
use std::io::{self};
use std::string::ToString;

use std::{thread, time};

use thiserror::Error;

// A succinct and useful guide to custom error handling:
// https://kazlauskas.me/entries/errors.html
// less succinct:
// http://www.sheshbabu.com/posts/rust-error-handling/

// An advantage of Rust is strongly typed applications.
// To make that a reality, have an intermediate step between the matching
// and the actual application code.
// This is where we extract the command line options into a struct.
// This isolates parsing logic, and the compiler helps everywhere else.
#[derive(Debug, PartialEq)]
pub struct CliArgs {
    path: String,
    pattern: String,
    version: String,
}

// This is what structopt does.
// It is done directly with clap to use the clap YAML file.
// This pattern is extended from:
// https://www.fpcomplete.com/rust/command-line-parsing-clap/
impl CliArgs {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self::new_from(std::env::args_os().into_iter())?)
    }
    pub fn new_from<I, T>(args: I) -> Result<Self, anyhow::Error>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        // CLI arguments
        let yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(yaml).get_matches_from_safe(args)?;
        let path = matches
            .value_of("path")
            .expect("This can't be None, it is required.");
        let pattern = matches
            .value_of("pattern")
            .expect("This can't be None, it is required.");
        // Validate inputs
        let valid_pattern = match pattern.is_empty() {
            true => {
                bail!("Invalid pattern.")
            }
            false => pattern,
        };
        Ok(CliArgs {
            path: path.to_string(),
            pattern: valid_pattern.to_string(),
            version: crate_version!().to_string(),
        })
    }
}

// Run the CLI code. This is called by main()
pub fn run() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = CliArgs::new()?;

    let stdout = io::stdout();
    let handle = io::BufWriter::new(stdout.lock());
    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("could not read file `{:?}`", &args.path))?;

    let pb  = setup_progress_spinner()?;

    thread::sleep(time::Duration::from_secs(5));

    find_matches(&content, &args.pattern, handle).with_context(|| {
        format!(
            "Unable to find pattern {} in file {}",
            &args.pattern, &args.path
        )
    })?;

    // Mark the progress bar as finished.
    pb.finish();
    Ok(())
}

fn setup_progress_spinner() -> Result<indicatif::ProgressBar, anyhow::Error> {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(120);
    pb.set_style(
        ProgressStyle::default_spinner()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ])
            .template("{spinner:.white} {msg}"),
    );
    pb.set_message("Inspecting...");
    Ok(pb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_find_matches() {
        let mut output = Vec::new();
        let _result = find_matches("lorem ipsum\ndolor sit amet", "lorem", &mut output);
        assert_eq!(output, b"lorem ipsum\n");
    }
}

/// Search for a pattern in a multi-line string.
//  Display the lines that contain it.
pub fn find_matches(
    content: &str,
    pattern: &str,
    mut writer: impl std::io::Write,
) -> Result<(), anyhow::Error> {
    for line in content.lines() {
        if line.contains(pattern) {
            writeln!(writer, "{}", line)
                .with_context(|| format!("Writing match result to stdout: {}", line))?;
        }
    }
    Ok(())
}
