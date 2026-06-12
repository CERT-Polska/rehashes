use super::{Result, blockhash, constants, error::Error, roll};
use serde::{Deserialize, Serialize};

/// The fuzzy hasher
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Hasher {
    bh_start: u32,
    bh_end: u32,
    bh: Vec<blockhash::Context>,
    total_size: u64,
    roll: roll::Roll,
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher {
    /// Build a new fuzzy hasher
    pub fn new() -> Hasher {
        let mut h = Hasher {
            bh_start: 0,
            bh_end: 1,
            bh: vec![blockhash::Context::new(); constants::NUM_BLOCKHASHES as usize],
            total_size: 0,
            roll: roll::Roll::new(),
        };
        h.bh[0].reset(true);
        h
    }

    fn memcpy_eliminate_sequences() -> usize {
        // TODO
        0
    }

    fn try_fork_blockhash(&mut self) {
        if self.bh_end < constants::NUM_BLOCKHASHES {
            self.bh[self.bh_end as usize].h = self.bh[(self.bh_end - 1) as usize].h;
            self.bh[self.bh_end as usize].half_h = self.bh[(self.bh_end - 1) as usize].half_h;

            self.bh[self.bh_end as usize].digest[0] = 0;
            self.bh[self.bh_end as usize].half_digest = 0;
            self.bh[self.bh_end as usize].d_len = 0;
            self.bh_end += 1;
        } else if self.bh_end == constants::NUM_BLOCKHASHES - 1 {
            self.bh[self.bh_end as usize].h = self.bh[(self.bh_end - 1) as usize].h;
        }
    }

    fn try_reduce_blockhash(&mut self) {
        if self.bh_end - self.bh_start < 2 {
            return;
        }

        if (u64::from(constants::MIN_BLOCK_SIZE) << self.bh_start) * u64::from(constants::SPAM_SUM_LENGTH)
            >= self.total_size
        {
            return;
        }

        if self.bh[(self.bh_start + 1) as usize].d_len < constants::SPAM_SUM_LENGTH / 2 {
            return;
        }

        self.bh_start += 1;
    }

    fn engine_step(&mut self, c: u8) {
        self.roll.hash(c);
        let h = self.roll.sum();
        for i in self.bh_start..self.bh_end {
            self.bh[i as usize].hash(c);
        }

        let mut j = self.bh_start;
        while j < self.bh_end {
            if h % (constants::MIN_BLOCK_SIZE << j) != (constants::MIN_BLOCK_SIZE << j) - 1 {
                break;
            }

            if self.bh[j as usize].d_len == 0 {
                self.try_fork_blockhash();
            }
            let pos = self.bh[j as usize].d_len as usize;
            self.bh[j as usize].digest[pos] =
                constants::get_base64_char((self.bh[j as usize].h % 64) as usize);
            self.bh[j as usize].half_digest =
                constants::get_base64_char((self.bh[j as usize].half_h % 64) as usize);

            if self.bh[j as usize].d_len < constants::SPAM_SUM_LENGTH - 1 {
                self.bh[j as usize].reset(false);
            } else {
                self.try_reduce_blockhash();
            }
            j += 1;
        }
    }

    /// Add data to the `Hasher`.
    pub fn update(&mut self, buffer: &[u8], len: usize) {
        self.total_size += len as u64;
        for item in buffer.iter().take(len) {
            self.engine_step(*item);
        }
    }

    /// Compute the hash of the data and return a `String` representation
    #[allow(clippy::too_many_lines)]
    pub fn digest(&mut self, flags: constants::Modes) -> Result<String> {
        let mut result = vec![0; constants::MAX_RESULT_LENGTH as usize];
        let mut pos = 0;
        let mut bi = self.bh_start;
        let mut h = self.roll.sum();

        while (u64::from(constants::MIN_BLOCK_SIZE) << bi) * u64::from(constants::SPAM_SUM_LENGTH) < self.total_size {
            bi += 1;
            if bi >= constants::NUM_BLOCKHASHES {
                return Err(Error::TooManyBlocks);
            }
        }

        while bi >= self.bh_end {
            bi -= 1;
        }

        while bi > self.bh_start && self.bh[bi as usize].d_len < constants::SPAM_SUM_LENGTH / 2 {
            bi -= 1;
        }

        let actual_blocksize = constants::MIN_BLOCK_SIZE << bi;
        let blocksize_string = actual_blocksize.to_string();
        let blocksize_chars = blocksize_string.into_bytes();
        let mut i = blocksize_chars.len();

        result[pos..(i + pos)].clone_from_slice(&blocksize_chars[..i]);
        result[i] = b':';
        i += 1;

        pos += i;
        i = self.bh[bi as usize].d_len as usize;

        match flags {
            constants::Modes::EliminateSequences => {
                i = Hasher::memcpy_eliminate_sequences();
            }
            _ => {
                result[pos..(i + pos)].clone_from_slice(&self.bh[bi as usize].digest[..i]);
            }
        }

        pos += i;
        if h != 0 {
            let base64val = constants::get_base64_char((self.bh[bi as usize].h % 64) as usize);
            result[pos] = base64val;
            if !matches!(flags, constants::Modes::EliminateSequences)
                || i < 3
                || base64val != result[pos - 1]
                || base64val != result[pos - 2]
                || base64val != result[pos - 3]
            {
                pos += 1;
            }
        } else if self.bh[bi as usize].digest[i] != 0 {
            let base64val = self.bh[bi as usize].digest[i];
            result[pos] = base64val;
            if !matches!(flags, constants::Modes::EliminateSequences)
                || i < 3
                || base64val != result[pos - 1]
                || base64val != result[pos - 2]
                || base64val != result[pos - 3]
            {
                pos += 1;
            }
        }
        result[pos] = b':';
        pos += 1;

        if bi < self.bh_end - 1 {
            bi += 1;
            i = self.bh[bi as usize].d_len as usize;

            if !matches!(flags, constants::Modes::DoNotTruncate)
                && i > ((constants::SPAM_SUM_LENGTH / 2) - 1) as usize
            {
                i = ((constants::SPAM_SUM_LENGTH / 2) - 1) as usize;
            }

            match flags {
                constants::Modes::EliminateSequences => {
                    i = Hasher::memcpy_eliminate_sequences();
                }
                _ => {
                    result[pos..(i + pos)].clone_from_slice(&self.bh[bi as usize].digest[..i]);
                }
            }
            pos += i;

            if h != 0 {
                h = match flags {
                    constants::Modes::DoNotTruncate => self.bh[bi as usize].h,
                    _ => self.bh[bi as usize].half_h,
                };
                let base64val = constants::get_base64_char((h % 64) as usize);
                result[pos] = base64val;
                if !matches!(flags, constants::Modes::EliminateSequences)
                    || i < 3
                    || base64val != result[pos - 1]
                    || base64val != result[pos - 2]
                    || base64val != result[pos - 3]
                {
                    pos += 1;
                }
            } else {
                i = match flags {
                    constants::Modes::DoNotTruncate => {
                        self.bh[bi as usize].digest[self.bh[bi as usize].d_len as usize]
                    }
                    _ => self.bh[bi as usize].half_digest,
                } as usize;

                if i != 0 {
                    result[pos] = i as u8;
                    if !matches!(flags, constants::Modes::EliminateSequences)
                        || i < 3
                        || i != result[pos - 1] as usize
                        || i != result[pos - 2] as usize
                        || i != result[pos - 3] as usize
                    {
                        pos += 1;
                    }
                }
            }
        } else if h != 0 {
            result[pos] = constants::get_base64_char((self.bh[bi as usize].h % 64) as usize);
        }
        unsafe {
            result.set_len(pos);
        }

        String::from_utf8(result).map_err(Error::InvalidHashString)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_increments_total_size() {
        let mut h = Hasher::new();
        let data = b"hello world";
        h.update(data, data.len());
        assert_eq!(h.total_size, data.len() as u64);
    }

    #[test]
    fn update_accumulates_total_size() {
        let mut h = Hasher::new();
        h.update(b"hello", 5);
        h.update(b"world", 5);
        assert_eq!(h.total_size, 10);
    }

    #[test]
    fn update_empty_does_not_change_size() {
        let mut h = Hasher::new();
        h.update(b"", 0);
        assert_eq!(h.total_size, 0);
    }

    #[test]
    fn serialize_deserialize_roundtrip_new() {
        let h = Hasher::new();
        let encoded = serde_cbor::to_vec(&h).expect("serialize failed");
        let decoded: Hasher = serde_cbor::from_slice(&encoded).expect("deserialize failed");
        assert_eq!(h.total_size, decoded.total_size);
        assert_eq!(h.bh_start, decoded.bh_start);
        assert_eq!(h.bh_end, decoded.bh_end);
    }

    #[test]
    fn serialize_deserialize_roundtrip_after_update() {
        let mut h = Hasher::new();
        h.update(b"the quick brown fox", 19);
        let encoded = serde_cbor::to_vec(&h).expect("serialize failed");
        let decoded: Hasher = serde_cbor::from_slice(&encoded).expect("deserialize failed");
        assert_eq!(h.total_size, decoded.total_size);
        assert_eq!(h.bh_start, decoded.bh_start);
        assert_eq!(h.bh_end, decoded.bh_end);
        assert_eq!(h.bh[0].h, decoded.bh[0].h);
        assert_eq!(h.bh[0].half_h, decoded.bh[0].half_h);
        assert_eq!(h.bh[0].d_len, decoded.bh[0].d_len);
    }

    #[test]
    fn digest_hello_world() {
        let input = b"Hello, world!";
        let mut h = Hasher::new();
        h.update(input, input.len());
        let result = h.digest(constants::Modes::None).expect("digest failed");
        assert_eq!(result, "3:a6/E:asE");
    }

    #[test]
    fn digest_hello_world_serialized_between_updates() {
        let mut h = Hasher::new();
        h.update(b"Hello, ", 7);
        let encoded = serde_cbor::to_vec(&h).expect("serialize failed");
        let mut h: Hasher = serde_cbor::from_slice(&encoded).expect("deserialize failed");
        h.update(b"world!", 6);
        let result = h.digest(constants::Modes::None).expect("digest failed");
        assert_eq!(result, "3:a6/E:asE");
    }

    #[test]
    fn deserialize_then_continue_update_matches_original() {
        let mut h1 = Hasher::new();
        h1.update(b"first chunk", 11);

        let encoded = serde_cbor::to_vec(&h1).expect("serialize failed");
        let mut h2: Hasher = serde_cbor::from_slice(&encoded).expect("deserialize failed");

        h1.update(b"second chunk", 12);
        h2.update(b"second chunk", 12);

        assert_eq!(h1.total_size, h2.total_size);
        assert_eq!(h1.bh[0].h, h2.bh[0].h);
    }
}
