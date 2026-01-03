use std::env;
use std::fmt;
use std::fs;
use std::io::{self, ErrorKind};
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
    for entry in fs::read_dir(path)? {
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

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::{self, File};
    use tempfile::NamedTempFile;
    use tempfile::tempdir;

    #[test]
    fn parse_args_user_input_none_returns_err() {
        let args = vec!["treer".to_string()];

        assert!(matches!(parse_args(&args), Err(AppError::InvalidArgs)));
    }

    #[test]
    fn parse_args_user_input_multiple_returns_err() {
        let args = vec!["treer".to_string(), "-a".to_string(), ".".to_string()];

        assert!(matches!(parse_args(&args), Err(AppError::InvalidArgs)));
    }

    #[test]
    fn parse_args_user_input_one_returns_ok() {
        let args = vec!["treer".to_string(), ".".to_string()];

        let path = parse_args(&args).unwrap();
        assert_eq!(path, PathBuf::from("."));
    }

    #[test]
    fn validate_path_existing_directory_returns_ok() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        let result = validate_path(path.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), path);
    }

    #[test]
    fn validate_path_nonexistent_path_returns_err() {
        let temp_dir = tempdir().unwrap();
        let non_existent_path = temp_dir.path().join("foo");

        let result = validate_path(non_existent_path);
        assert!(matches!(result, Err(AppError::PathNotFound)));
    }

    #[test]
    fn validate_path_file_returns_err() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let result = validate_path(path);
        assert!(matches!(result, Err(AppError::NotADirectory)));
    }

    #[test]
    #[ignore]
    fn validate_path_permission_denied_returns_err() {}

    #[test]
    fn read_directory_empty_directory_returns_ok() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        let entries = read_directory(&path).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn read_directory_with_file_returns_ok() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        // ファイルを作成
        File::create(path.join("file1.txt")).unwrap();
        File::create(path.join("file2.txt")).unwrap();

        let entries = read_directory(&path).unwrap();
        let names: Vec<_> = entries
            .iter()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        assert!(names.contains(&"file1.txt".to_string()));
        assert!(names.contains(&"file2.txt".to_string()));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn read_directory_with_subdirectories_returns_ok() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        // サブディレクトリを作成
        fs::create_dir(path.join("sub1")).unwrap();
        fs::create_dir(path.join("sub2")).unwrap();

        let entries = read_directory(&path).unwrap();
        let names: Vec<_> = entries
            .iter()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        assert!(names.contains(&"sub1".to_string()));
        assert!(names.contains(&"sub2".to_string()));
        assert_eq!(names.len(), 2);
    }
}
