use std::hash::Hash;
use std::io::Write;

use crypto_hash::{Algorithm, Hasher};

pub fn hash<H>(data: H) -> String
where
    H: Hash,
{
    let mut hasher = Sha256(Hasher::new(Algorithm::SHA256));
    data.hash(&mut hasher);
    hasher.finish()
}

struct Sha256(Hasher);

impl Sha256 {
    fn finish(&mut self) -> String {
        hex::encode(&self.0.finish()[..16])
    }
}

impl std::hash::Hasher for Sha256 {
    fn write(&mut self, bytes: &[u8]) {
        let _ = self.0.write_all(bytes);
    }

    fn finish(&self) -> u64 {
        unreachable!()
    }
}
