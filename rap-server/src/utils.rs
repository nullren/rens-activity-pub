use base64::engine::general_purpose;
use base64::Engine;
pub fn base64_decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let decoded = general_purpose::STANDARD.decode(data)?;
    Ok(decoded)
}
