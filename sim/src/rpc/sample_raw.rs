pub struct RawImageZip {
    pub width: u16,
    pub height: u16,
    pub zip_data: &'static [u8],
}

pub static RAW_IMAGE_ZIP: RawImageZip = RawImageZip {
    width: 6248,
    height: 4176,
    zip_data: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/rpc/sample_raw.zip"
    )),
};
