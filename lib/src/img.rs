use super::ASIAir;
use std::io::Cursor;
use std::io::Read;
use zip::ZipArchive;

impl ASIAir {
    pub async fn get_current_img(
        &mut self,
    ) -> Result<(Vec<u8>, u16, u16), Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_current_img";
        let result = self.rpc_request_4800(method, None).await?;

        let cursor = Cursor::new(&result.data);
        let mut archive = ZipArchive::new(cursor)?;

        // Assuming you want the first file in the archive
        if archive.len() == 0 {
            return Err("Zip archive is empty".into());
        }
        let mut file = archive.by_index(0)?;
        let mut extracted_data = Vec::new();
        file.read_to_end(&mut extracted_data)?;

        return Ok((extracted_data, result.width, result.height));
    }
}
