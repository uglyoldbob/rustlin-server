use crate::Exception;
use des::cipher::BlockDecryptMut;
use des::cipher::KeyInit;
use omnom::ReadExt;
use std::collections::HashMap;
use std::io::Read;
use std::io::Seek;

/// A helper trait for reading raw file data from a Read object
trait VecReader
where
    Self: Read,
{
    /// Reads a vector of data from a buffer, the vector must already have the length desired to read
    fn read_vec(&mut self, buf: &mut Vec<u8>) -> Result<(), ()>;
}

impl VecReader for std::fs::File {
    fn read_vec(&mut self, buf: &mut Vec<u8>) -> Result<(), ()> {
        buf.clear();
        let mut partial: [u8; 32] = [0; 32];
        let mut remaining = buf.capacity();
        let mut done = false;
        loop {
            if remaining == 0 {
                done = true;
                break;
            }
            let count = if remaining > 32 {
                self.read(&mut partial[..])
            } else {
                self.read(&mut partial[0..remaining])
            };
            match count {
                Ok(n) => {
                    for b in partial[..n].iter() {
                        buf.push(*b);
                    }
                    remaining -= n;
                }
                Err(_e) => {
                    break;
                }
            }
        }
        if done {
            return Ok(());
        } else {
            return Err(());
        }
    }
}

/// This structure describes a single pack file used as a container for
/// game resources in the client.
pub struct Pack {
    encrypted: bool,
    name: String,
    file_data: HashMap<String, FileEntry>,
    contents: Option<std::fs::File>,
}

#[derive(Clone)]
struct FileEntry {
    offset: u32,
    size: u32,
}

fn des_decrypt(key: String, data: &mut Vec<u8>) {
    let key = crypto_common::generic_array::GenericArray::from_slice(key[..].as_bytes());
    let mut key = des::Des::new(key);
    for chunk in data.chunks_exact_mut(8) {
        let data = crypto_common::generic_array::GenericArray::from_mut_slice(&mut chunk[..]);
        key.decrypt_block_mut(data);
    }
}

impl Pack {
    pub fn new(n: String, e: bool) -> Self {
        Self {
            encrypted: e,
            name: n,
            file_data: HashMap::new(),
            contents: None,
        }
    }

    pub fn file_extensions(&self) -> HashMap<String, u32> {
        let mut hm = HashMap::new();
        for key in self.file_data.keys() {
            let extension = key.split('.').nth(1);
            if let Some(extension) = extension {
                let extension = extension.to_string();
                if hm.contains_key(&extension) {
                    let val = hm.get_mut(&extension).unwrap();
                    *val += 1;
                } else {
                    hm.insert(extension, 1);
                }
            }
        }
        hm
    }

    fn get_file_index(&self, name: String) -> Option<FileEntry> {
        self.file_data.get(&name).cloned()
    }

    pub fn raw_file_contents(&mut self, name: String) -> Option<Vec<u8>> {
        let index = self.get_file_index(name.clone());
        if let Some(f) = &mut self.contents {
            if let Some(i) = index {
                let offset = i.offset;
                let size = i.size;
                if let Err(_e) = f.seek(std::io::SeekFrom::Start(offset as u64)) {
                    return None;
                }
                let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
                let val = f.read_vec(&mut buffer);
                if let Err(_e) = val {
                    return None;
                }
                Some(buffer)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn decrypted_file_contents(&mut self, name: String) -> Option<Vec<u8>> {
        let mut file = self.raw_file_contents(name);
        if let Some(f) = &mut file {
            des_decrypt("~!@#%^$<".to_string(), f);
            Some(f.to_vec())
        } else {
            None
        }
    }

    pub fn load(&mut self) -> Result<(), Exception> {
        let content = format!("{}.pak", self.name);
        let index = format!("{}.idx", self.name);
        let contents = std::fs::File::open(content)?;
        self.contents = Some(contents);
        if let Some(contents) = &self.contents {
            let mut indx: std::fs::File = std::fs::File::open(index)?;
            let size32: u32 = indx.read_le()?;
            let size = size32 as u64;
            let size2 = (indx.metadata()?.len() - 4) / 28;
            let content_size = contents.metadata()?.len() as u64;
            if size != size2 {
                println!(
                    "File size mismatch {:x} {:x} for {}",
                    size, size2, self.name
                );
                return Err(Exception::ContentError);
            }
            let mut index_contents = Vec::new();
            indx.read_to_end(&mut index_contents)?;
            if self.encrypted {
                des_decrypt("~!@#%^$<".to_string(), &mut index_contents);
            }
            let mut indx = std::io::Cursor::new(index_contents);
            for _i in 0..size {
                let offset: u32 = indx.read_le()?;
                let mut name: [u8; 20] = [0; 20];
                indx.read_exact(&mut name[..])?;
                let size: u32 = indx.read_le()?;
                let mut name = String::from_utf8_lossy(&name[..]).into_owned();
                name.make_ascii_lowercase();
                name = name.trim_matches(char::from(0)).to_string();
                self.file_data.insert(
                    name,
                    FileEntry {
                        offset: offset,
                        size: size,
                    },
                );
                if offset as u64 + size as u64 > content_size as u64 {
                    println!("Invalid entry");
                    return Err(Exception::ContentError);
                }
            }
        } else {
            return Err(Exception::IoError(std::io::Error::other("No contents in pack?".to_string()))); //probably
        }
        Ok(())
    }
}
