// Protocol Message Types
pub const MSG_TYPE_REQUEST: u8 = 0x00;
pub const MSG_TYPE_EXECUTION_WITNESS: u8 = 0x01;

pub const HEADER_SIZE: usize = 9; // 1 byte type + 8 bytes length
pub const MAX_PAYLOAD_SIZE: u64 = 5 * 1024 * 1024 * 1024; // 5 GB

pub fn pack_header(msg_type: u8, length: u64) -> [u8; HEADER_SIZE] {
    let mut header = [0u8; HEADER_SIZE];
    header[0] = msg_type;
    header[1..9].copy_from_slice(&length.to_be_bytes());
    header
}

pub fn unpack_header(header: &[u8; 9]) -> (u8, u64) {
    let msg_type = header[0];
    let mut len_bytes = [0u8; 8];
    len_bytes.copy_from_slice(&header[1..9]);
    let len = u64::from_be_bytes(len_bytes);
    (msg_type, len)
}
