use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Debug)]
enum AppError {
    InvalidArgs,
    PathNotFound,
    NotADirectory,
    Io(io::Error),
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidArgs => write!(f, "invalid arguments"),
            AppError::PathNotFound => write!(f, "path not found"),
            AppError::NotADirectory => write!(f, "not a directory"),
            AppError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

fn map_metadata_error(e: io::Error) -> AppError {
    match e.kind() {
        ErrorKind::NotFound => AppError::PathNotFound,
        ErrorKind::PermissionDenied => AppError::Io(e),
        _ => AppError::Io(e),
    }
}

fn validate_path(path: PathBuf) -> Result<PathBuf, AppError> {
    let metadata = fs::metadata(&path).map_err(map_metadata_error)?;

    if !metadata.is_dir() {
        return Err(AppError::NotADirectory);
    }

    Ok(path)
}

fn read_directory(path: &PathBuf) -> Result<Vec<fs::DirEntry>, AppError> {
    let mut entries_vec = Vec::new();

    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        entries_vec.push(entry);
    }

    Ok(entries_vec)
}

fn parse_args(args: &[String]) -> Result<PathBuf, AppError> {
    let count = args.len();
    match count {
        2 => Ok(args[1].clone().into()),
        _ => Err(AppError::InvalidArgs),
    }
}

fn run() -> Result<(), AppError> {
    let args: Vec<String> = env::args().collect();
    let path = parse_args(&args)?;
    let path = validate_path(path)?;
    let entries = read_directory(&path)?;

    for entry in entries {
        println!("{}", entry.file_name().to_string_lossy());
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
    }
}
