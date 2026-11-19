//! Command line tool for the dungeon generator.
//! Creates a floor and saves it to a json file, like `main.py` of the Python generator.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use dungeon_generator::Generator;
use dungeon_generator::utils::constants::{
    DEFAULT_FLOOR_DIRECTORY, DEFAULT_FLOOR_NAME, JSON_SUFFIX,
};

#[derive(Parser, Debug)]
#[command(about = "Creates a dungeon floor and saves it to a json file.")]
struct Args {
    /// Seed for the random number generator, default is a random seed
    #[arg(short, long)]
    seed: Option<String>,

    /// Output file name, a number based on the current time is appended
    #[arg(short, long, default_value = DEFAULT_FLOOR_NAME)]
    output: String,

    /// Id for the floor, must be 0 or greater. Higher floor ids get more rooms
    #[arg(
        short,
        long = "floor",
        default_value_t = 2,
        value_parser = parse_floor_id,
        allow_negative_numbers = true
    )]
    floor: u32,
}

/// Parses a floor id with the messages of the Python version.
fn parse_floor_id(value: &str) -> Result<u32, String> {
    let floor_id: i64 = value
        .parse()
        .map_err(|_| format!("{value} is not a valid number"))?;
    if floor_id < 0 {
        return Err("the floor id must be 0 or greater".to_owned());
    }
    u32::try_from(floor_id).map_err(|_| format!("the floor id must be at most {}", u32::MAX))
}

/// Creates a random seed of 32 hex characters, like `secrets.token_hex(16)` in Python.
fn random_seed() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("the operating system provides random numbers");
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// File name with a number based on the current time, e.g. `floor123456.json`.
fn output_file_name(output: &str) -> String {
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.subsec_micros())
        .unwrap_or(0);
    format!("{output}{micros}{JSON_SUFFIX}")
}

/// Generates the floor and saves it in the output folder, which is created if needed.
/// Returns the path of the written file.
fn generate_and_save(
    seed: &str,
    floor_id: u32,
    output: &str,
    output_folder: &Path,
) -> io::Result<PathBuf> {
    if !output_folder.exists() {
        fs::create_dir_all(output_folder)?;
        println!("No output folder found. Creating specific folder");
    }
    let mut generator = Generator::new(seed, floor_id);
    generator.generate();
    let path = output_folder.join(output_file_name(output));
    generator.save(&path)?;
    Ok(path)
}

fn main() -> ExitCode {
    let args = Args::parse();
    let seed = args
        .seed
        .filter(|seed| !seed.is_empty())
        .unwrap_or_else(random_seed);
    match generate_and_save(
        &seed,
        args.floor,
        &args.output,
        Path::new(DEFAULT_FLOOR_DIRECTORY),
    ) {
        Ok(path) => {
            println!("Floor saved to {}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: could not save the floor: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dungeon_generator::Floor;

    fn parse(args: &[&str]) -> Result<Args, clap::Error> {
        Args::try_parse_from(std::iter::once("dungeon-cli").chain(args.iter().copied()))
    }

    #[test]
    fn uses_the_defaults_of_the_python_version() {
        let args = parse(&[]).unwrap();
        assert_eq!(args.seed, None);
        assert_eq!(args.output, "floor");
        assert_eq!(args.floor, 2);
    }

    #[test]
    fn parses_short_and_long_options() {
        let args = parse(&["-s", "my-seed", "-f", "3", "-o", "my-floor"]).unwrap();
        assert_eq!(args.seed.as_deref(), Some("my-seed"));
        assert_eq!(args.floor, 3);
        assert_eq!(args.output, "my-floor");
        let args = parse(&["--seed", "x", "--floor", "0", "--output", "y"]).unwrap();
        assert_eq!(
            (args.seed.as_deref(), args.floor, args.output.as_str()),
            (Some("x"), 0, "y")
        );
    }

    #[test]
    fn rejects_invalid_floor_ids() {
        let error = parse(&["-f", "-1"]).unwrap_err().to_string();
        assert!(
            error.contains("the floor id must be 0 or greater"),
            "{error}"
        );
        let error = parse(&["-f", "abc"]).unwrap_err().to_string();
        assert!(error.contains("abc is not a valid number"), "{error}");
    }

    #[test]
    fn random_seeds_are_hex_and_differ() {
        let seed = random_seed();
        assert_eq!(seed.len(), 32);
        assert!(seed.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(seed, random_seed());
    }

    #[test]
    fn saves_the_floor_in_a_new_folder() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("generation");
        let path = generate_and_save("abc", 3, "my-floor", &folder).unwrap();

        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(
            name.starts_with("my-floor") && name.ends_with(".json"),
            "{name}"
        );
        assert_eq!(path.parent(), Some(folder.as_path()));

        let saved = Floor::from_json(&fs::read_to_string(&path).unwrap()).unwrap();
        let mut generator = Generator::new("abc", 3);
        assert_eq!(saved.rooms(), generator.generate().rooms());
        assert_eq!(saved.seed(), "abc");
        assert_eq!(saved.floor_id(), Some(3));
    }
}
