use anyhow::{Context, Result};
use md5::{Digest, Md5};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

pub fn compute_md5<R: Read>(mut reader: R) -> Result<String> {
    let mut hasher = Md5::new();
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn process_file(path: &PathBuf, binary: bool) -> Result<()> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let md5sum = compute_md5(file)?;
    let mode = if binary { "*" } else { " " };
    println!("{}{}{}", md5sum, mode, path.display());
    Ok(())
}

pub fn verify_checksums(path: &PathBuf) -> Result<()> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 2 {
            eprintln!("{}: improperly formatted checksum line", path.display());
            continue;
        }

        let (expected_sum, filename) = (parts[0], parts[1]);
        let file_path = PathBuf::from(filename);

        if let Ok(file) = File::open(&file_path) {
            let computed_sum = compute_md5(file)?;
            if computed_sum == expected_sum {
                println!("{}: OK", filename);
            } else {
                println!("{}: FAILED", filename);
            }
        } else {
            eprintln!("{}: No such file or directory", filename);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use tempfile::tempdir;

    fn create_test_file(content: &[u8]) -> Result<(PathBuf, String)> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path)?;
        file.write_all(content)?;
        Ok((file_path, dir.into_path().to_string_lossy().to_string()))
    }

    #[test]
    fn test_compute_md5() -> Result<()> {
        let content = b"Hello, World!";
        let (file_path, _temp_dir) = create_test_file(content)?;

        let file = File::open(&file_path)?;
        let md5sum = compute_md5(file)?;

        // 预期的MD5值是"Hello, World!"字符串的MD5校验和
        assert_eq!(md5sum, "65a8e27d8879283831b664bd8b7f0ad4");
        Ok(())
    }

    #[test]
    fn test_process_file() -> Result<()> {
        let content = b"Test content";
        let (file_path, _temp_dir) = create_test_file(content)?;

        // 测试文本模式
        process_file(&file_path, false)?;

        // 测试二进制模式
        process_file(&file_path, true)?;

        Ok(())
    }

    #[test]
    fn test_verify_checksums() -> Result<()> {
        let dir = tempdir()?;

        // 创建一个测试文件
        let test_file_path = dir.path().join("test.txt");
        std::fs::write(&test_file_path, "Test content")?;

        // 创建校验和文件
        let checksum_file_path = dir.path().join("checksums.md5");
        let file = File::open(&test_file_path)?;
        let md5sum = compute_md5(file)?;

        std::fs::write(
            &checksum_file_path,
            format!("{} {}", md5sum, test_file_path.to_string_lossy()),
        )?;

        // 验证校验和
        verify_checksums(&checksum_file_path)?;

        Ok(())
    }
}
