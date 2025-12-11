pub(crate) fn search(buf: &[u8], pattern: &[u8], start_index: usize) -> Option<usize> {
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
