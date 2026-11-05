use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

const STACK_FILE: &str = "stack.txt";

pub fn default_path() -> PathBuf {
    PathBuf::from(STACK_FILE)
}

/// Loads one Value per line, skipping lines that don't parse. A missing file is an empty Stack.
pub fn load(path: &Path) -> io::Result<Vec<f64>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    Ok(BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| line.parse::<f64>().ok())
        .collect())
}

/// Writes one Value per line, creating the parent directory if it doesn't exist.
pub fn save(path: &Path, stack: &[f64]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = BufWriter::new(File::create(path)?);
    for value in stack {
        writeln!(file, "{value}")?;
    }
    file.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn saved_stack_loads_back() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("stack.txt");
        let stack = [1.0, -2.5, 1e300, f64::INFINITY, std::f64::consts::PI];
        save(&path, &stack).unwrap();
        assert_eq!(load(&path).unwrap(), stack);
    }

    #[test]
    fn missing_file_loads_as_empty_stack() {
        let dir = tempdir().unwrap();
        assert_eq!(
            load(&dir.path().join("missing.txt")).unwrap(),
            Vec::<f64>::new()
        );
    }

    #[test]
    fn corrupt_lines_are_skipped() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("stack.txt");
        std::fs::write(&path, "1\nnot a number\n\n2.5\n").unwrap();
        assert_eq!(load(&path).unwrap(), vec![1.0, 2.5]);
    }

    #[test]
    fn save_overwrites_previous_contents() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("stack.txt");
        save(&path, &[1.0, 2.0]).unwrap();
        save(&path, &[3.0]).unwrap();
        assert_eq!(load(&path).unwrap(), vec![3.0]);
    }

    #[test]
    fn save_creates_missing_parent_directories() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("rpn-calc").join("stack.txt");
        save(&path, &[4.0, 2.0]).unwrap();
        assert_eq!(load(&path).unwrap(), vec![4.0, 2.0]);
    }

    #[test]
    fn default_path_is_stack_txt_in_current_directory() {
        assert_eq!(default_path(), PathBuf::from("stack.txt"));
    }
}
