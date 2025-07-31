#[derive(Clone)]
pub struct StringTable {
    data: Vec<String>,
}

impl From<Vec<u8>> for StringTable {
    fn from(value: Vec<u8>) -> Self {
        let ascii = std::str::from_utf8(&value).unwrap();
        let lines: Vec<&str> = ascii.split("\r\n").collect();
        let count = lines[0].parse::<u32>().unwrap();
        let mut strings = Vec::with_capacity(count as usize);
        println!("There are {} entries", count);
        for l in lines.iter().skip(1) {
            strings.push(l.to_owned().to_string());
        }
        Self { data: strings }
    }
}

impl StringTable {
    /// Get the specified index string
    pub fn get(&self, i: usize) -> Option<&String> {
        self.data.get(i)
    }
}
