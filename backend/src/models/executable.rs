use serde::Serialize;

#[derive(Default, Clone, Debug)]
pub struct Executable {
    pub data: Vec<u8>, // the raw data of the executable
    pub filename: String,
    pub name: String,      // the name before the extension
    pub extension: String, // may be empty string
    pub key_start: usize,  // the index of the byte where the key starts
    pub key_end: usize,    // the index of the byte where the key ends
}

impl Executable {
    pub fn with_key(&self, new_key: &[u8]) -> Vec<u8> {
        let mut data = self.data.clone();

        // Copy the key into the data
        for i in 0..new_key.len() {
            data[self.key_start + i] = new_key[i];
        }

        // If the new key is shorter than the old key, we just write over the remaining data
        if new_key.len() < self.key_end - self.key_start {
            for item in data
                .iter_mut()
                .take(self.key_end)
                .skip(self.key_start + new_key.len())
            {
                *item = b' ';
            }
        }

        data
    }
}

#[derive(Debug, Serialize)]
pub struct ExecutableJson {
    pub id: String,
    pub size: usize,
    pub filename: String,
}
