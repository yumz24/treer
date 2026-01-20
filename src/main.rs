use std::env;
use std::fmt;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum AppError {
    InvalidArgs,
    PathNotFound(PathBuf),
    NotADirectory(PathBuf),
    PermissionDenied(PathBuf),
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
            AppError::PathNotFound(path) => write!(f, "path not found: {}", path.display()),
            AppError::NotADirectory(path) => write!(f, "not a directory: {}", path.display()),
            AppError::PermissionDenied(path) => write!(f, "permission denied: {}", path.display()),
            AppError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

fn validate_path<P: AsRef<Path>>(path: P) -> Result<(), AppError> {
    let path_ref = path.as_ref();

    let metadata = fs::metadata(path_ref).map_err(|e| match e.kind() {
        ErrorKind::NotFound => AppError::PathNotFound(path_ref.to_path_buf()),
        _ => AppError::Io(e),
    })?;

    if !metadata.is_dir() {
        return Err(AppError::NotADirectory(path_ref.to_path_buf()));
    }

    Ok(())
}

fn read_directory<P: AsRef<Path>>(path: P) -> Result<Vec<fs::DirEntry>, AppError> {
    let path_ref = path.as_ref();
    fs::read_dir(path_ref)
        .map_err(|e| match e.kind() {
            ErrorKind::PermissionDenied => AppError::PermissionDenied(path_ref.to_path_buf()),
            _ => AppError::Io(e),
        })?
        .map(|res| {
            res.map_err(AppError::from)
        })
        .collect()
}

fn parse_args(args: &[String]) -> Result<PathBuf, AppError> {
    let count = args.len();
    match count {
        2 => Ok(PathBuf::from(&args[1])),
        _ => Err(AppError::InvalidArgs),
    }
}

fn run() -> Result<(), AppError> {
    let args: Vec<String> = env::args().collect();
    let path = parse_args(&args)?;

    validate_path(&path)?;
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
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        let result = validate_path(path);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_path_nonexistent_path_returns_err() {
        let temp_dir = tempdir().unwrap();
        let non_existent_path = temp_dir.path().join("foo");

        let result = validate_path(non_existent_path);
        assert!(matches!(result, Err(AppError::PathNotFound(_))));
    }

    #[test]
    fn validate_path_file_returns_err() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let result = validate_path(path);
        assert!(matches!(result, Err(AppError::NotADirectory(_))));
    }

    // TODO: テストを動かす方法があれば作成する
    #[test]
    #[ignore]
    fn validate_path_permission_denied_returns_err() {}

    #[test]
    fn read_directory_empty_directory_returns_ok() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let entries = read_directory(path).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn read_directory_with_file_returns_ok() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // ファイルを作成
        File::create(path.join("file1.txt")).unwrap();
        File::create(path.join("file2.txt")).unwrap();

        let entries = read_directory(path).unwrap();
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
        let path = dir.path();

        fs::create_dir(path.join("sub1")).unwrap();
        fs::create_dir(path.join("sub2")).unwrap();

        let entries = read_directory(path).unwrap();
        let names: Vec<_> = entries
            .iter()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        assert!(names.contains(&"sub1".to_string()));
        assert!(names.contains(&"sub2".to_string()));
        assert_eq!(names.len(), 2);
    }
}
