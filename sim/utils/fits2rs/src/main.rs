use std::env;
use std::fs::{write, File};
use std::io::Cursor;
use std::io::{BufReader, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

use fitsrs::{Fits, Pixels, HDU};
use rayon::prelude::*;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input.fits> <output_dir> <module_name>", args[0]);
        std::process::exit(1);
    }

    let input_fits = &args[1];
    let output_dir = &args[2];
    let module_name = &args[3];

    let reader = BufReader::new(File::open(input_fits).map_err(|e| e.to_string())?);
    let mut hdu_list = Fits::from_reader(reader);

    if let Some(Ok(HDU::Primary(hdu))) = hdu_list.next() {
        let xtension = hdu.get_header().get_xtension();
        let width = *xtension.get_naxisn(1).ok_or("Missing NAXIS1")? as usize;
        let height = *xtension.get_naxisn(2).ok_or("Missing NAXIS2")? as usize;

        let image = hdu_list.get_data(&hdu);
        let Pixels::I16(data) = image.pixels() else {
            return Err("Expected I16 pixel data".to_string());
        };

        generate_raw_data_and_module(
            &data.collect::<Vec<i16>>(),
            width,
            height,
            output_dir,
            module_name,
        )
        .map_err(|e| e.to_string())?;

        println!(
            "✅ Generated `{}/{}.rs` and binary image file.",
            output_dir, module_name
        );
        Ok(())
    } else {
        Err("No primary HDU found".to_string())
    }
}

fn generate_raw_data_and_module(
    image_data: &[i16],
    width: usize,
    height: usize,
    out_dir: &str,
    module_name: &str,
) -> std::io::Result<()> {
    let zip_path = Path::new(out_dir).join(format!("{}.zip", module_name));
    let rs_path = Path::new(out_dir).join(format!("{}.rs", module_name));

    // Write raw data into a .zip file in-memory
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("raw_data", options)?;
        // Parallel conversion of i16 slice to Vec<u8>
        let raw_bytes: Vec<u8> = image_data
            .par_iter()
            .flat_map_iter(|&val| val.to_be_bytes())
            .collect();
        zip.write_all(&raw_bytes)?;

        zip.finish()?;
    }

    // Write .zip file to disk
    std::fs::write(&zip_path, buffer.into_inner())?;

    // Generate Rust module that embeds the .zip file
    let rust_code = format!(
        r#"
pub struct RawImageZip {{
    pub width: u16,
    pub height: u16,
    pub zip_data: &'static [u8],
}}

pub static RAW_IMAGE_ZIP: RawImageZip = RawImageZip {{
    width: {width},
    height: {height},
    zip_data: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/{module_name}.zip")),
}};
"#,
        width = width,
        height = height,
        module_name = module_name
    );

    write(&rs_path, rust_code)?;
    Ok(())
}
