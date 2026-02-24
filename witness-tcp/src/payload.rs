use std::sync::OnceLock;

pub const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks

static STATIC_PAYLOAD: OnceLock<Vec<u8>> = OnceLock::new();

pub fn get_chunk() -> &'static [u8] {
    STATIC_PAYLOAD.get_or_init(|| {
        let mut v = Vec::with_capacity(CHUNK_SIZE);
        for i in 0..CHUNK_SIZE {
            v.push((i % 256) as u8);
        }
        v
    })
}
