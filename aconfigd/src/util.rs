/*
 * Copyright (C) 2024 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use openssl::hash::{Hasher, MessageDigest};
use std::fs::File;
use std::io::Read;
use std::path::Path;

// The digest is returned as a hexadecimal string.
pub(crate) fn get_files_digest(paths: &[&Path]) -> Result<String, std::io::Error> {
    let mut hasher = Hasher::new(MessageDigest::sha256())?;
    let mut buffer = [0; 1024];
    for path in paths {
        let mut f = File::open(path)?;
        loop {
            let n = f.read(&mut buffer[..])?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer)?;
        }
    }
    let digest: &[u8] = &hasher.finish()?;
    let mut xdigest = String::new();
    for x in digest {
        xdigest.push_str(format!("{:02x}", x).as_str());
    }
    println!("{}", xdigest);
    Ok(xdigest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_get_files_digest() {
        let path1 = Path::new("/tmp/hi.txt");
        let path2 = Path::new("/tmp/bye.txt");
        let mut file1 = File::create(path1).unwrap();
        let mut file2 = File::create(path2).unwrap();
        file1.write_all(b"Hello, world!").expect("Writing to file");
        file2.write_all(b"Goodbye, world!").expect("Writing to file");
        let digest = get_files_digest(&[path1, path2]);
        assert_eq!(
            digest.expect("Calculating digest"),
            "8352c31d9ff5f446b838139b7f4eb5fed821a1f80d6648ffa6ed7391ecf431f4"
        );
    }
}
