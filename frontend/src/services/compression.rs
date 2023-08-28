
pub mod compression {
    use std::io::Write;

    pub fn compress(source: Vec<u8>, level: i32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut encoder = zstd::Encoder::new(Vec::new(), level)?;
        encoder.write_all(&source)?;
        let compressed = encoder.finish()?;
        Ok(compressed)
    }

    pub fn decompress(compressed: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut decoder = zstd::Decoder::new(&compressed[..])?;
        let mut decompressed = Vec::new();
        std::io::copy(&mut decoder, &mut decompressed)?;
        Ok(decompressed)
    }    

}