//! Stable content identity for the CLI source, shared by build.rs and doctor.
//! This is an alignment fingerprint, not an authenticity/signature mechanism.
use std::{
    fs, io,
    path::{Path, PathBuf},
};

fn source_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            source_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

pub fn inputs(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    source_files(&root.join("src"), &mut files)?;
    for name in ["Cargo.toml", "Cargo.lock", "build.rs"] {
        let path = root.join(name);
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

pub fn fingerprint(root: &Path) -> io::Result<String> {
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut feed = |bytes: &[u8]| {
        for byte in (bytes.len() as u64)
            .to_le_bytes()
            .iter()
            .chain(bytes.iter())
        {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
    };
    for file in inputs(root)? {
        let name = file
            .strip_prefix(root)
            .map_err(io::Error::other)?
            .to_string_lossy()
            .replace('\\', "/");
        feed(name.as_bytes());
        feed(&fs::read(file)?);
    }
    Ok(format!("fnv1a64:{hash:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_content_not_build_time_or_checkout_path_defines_identity() {
        let tmp = tempfile::TempDir::new().unwrap();
        for name in ["a", "b"] {
            let root = tmp.path().join(name);
            fs::create_dir_all(root.join("src")).unwrap();
            fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
            fs::write(root.join("Cargo.toml"), "[package]\nname='fixture'\n").unwrap();
        }
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        assert_eq!(fingerprint(&a).unwrap(), fingerprint(&b).unwrap());
        fs::write(b.join("src/main.rs"), "fn main() { panic!() }\n").unwrap();
        assert_ne!(fingerprint(&a).unwrap(), fingerprint(&b).unwrap());
        let before = fingerprint(&a).unwrap();
        fs::create_dir(a.join("target")).unwrap();
        fs::write(a.join("target/binary"), "built elsewhere").unwrap();
        assert_eq!(before, fingerprint(&a).unwrap());
    }
}
