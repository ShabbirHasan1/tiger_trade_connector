/// u32_to_vec_u8 converts u32 to a vector of four bytes since we need
/// the same prefix size for all messages
fn u32_to_vec_u8(value: u32) -> Vec<u8> {
    let byte1 = ((value >> 24) & 0xFF) as u8;
    let byte2 = ((value >> 16) & 0xFF) as u8;
    let byte3 = ((value >> 8) & 0xFF) as u8;
    let byte4 = (value & 0xFF) as u8;

    vec![byte1, byte2, byte3, byte4]
}

/// make_message converts a borrowed string slice to a vector of bytes
/// and prepends it with a size of that slice.
/// It will allow a client to properly read a message
pub fn make_message(text: &str) -> Vec<u8> {
    let message_size = text.len() as u32;
    let mut message = u32_to_vec_u8(message_size);
    message.extend(text.as_bytes());
    message
}

/// read_message converts an array of u8 to a borrowed sting slice
/// by properly reading a message size from the first 4 bytes
/// and then reading a message content
pub fn read_message(buf: &[u8]) -> &str {
    let message_size = u32::from_be_bytes(buf[0..4].try_into().unwrap());
    let message_usize = usize::try_from(message_size).unwrap();
    let message = std::str::from_utf8(&buf[4..4 + message_usize]).unwrap();
    message
}
