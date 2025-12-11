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
    pub fn search_pattern(buf: &[u8], pattern: &[u8], start_index: usize) -> Option<usize> {
        let mut i = start_index;

        // If the buffer is empty, the pattern is too long
        if pattern.len() > buf.len() {
            return None;
        }

        // If the pattern is empty
        if pattern.is_empty() {
            return None;
        }

        // If the starting index is too high
        if start_index >= buf.len() {
            return None;
        }

        while i < buf.len() {
            for j in 0..pattern.len() {
                // If the pattern is too long to fit in the buffer anymore
                if i + j >= buf.len() {
                    return None;
                }

                // If the pattern stops matching
                if buf[i + j] != pattern[j] {
                    break;
                }

                // If the pattern is found
                if j == pattern.len() - 1 {
                    return Some(i);
                }
            }

            i += 1;
        }
        None
    }

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
