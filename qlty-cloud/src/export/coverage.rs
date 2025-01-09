use crate::format::{GzFormatter, JsonEachRowFormatter, JsonFormatter};
use anyhow::{Context, Result};
use qlty_types::tests::v1::{CoverageMetadata, FileCoverage, ReportFile};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::{write::FileOptions, ZipWriter};

fn compress_files(files: Vec<String>, output_file: &str) -> Result<()> {
    // Create the output ZIP file
    let zip_file = File::create(output_file)?;
    let mut zip = ZipWriter::new(zip_file);

    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated) // Compression method
        .unix_permissions(0o755);

    // Iterate over the list of files to compress
    for file_path in files {
        let path = Path::new(&file_path);

        if path.is_file() {
            // Add the file to the archive
            let file_name = path.file_name().unwrap().to_string_lossy();
            zip.start_file(file_name, options)?;

            // Write the file content to the archive
            let mut file = File::open(path)?;
            std::io::copy(&mut file, &mut zip)?;
        } else {
            eprintln!("Skipping non-file: {}", file_path);
        }
    }

    // Finalize the ZIP file
    zip.finish()?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct CoverageExport {
    pub metadata: CoverageMetadata,
    pub report_files: Vec<ReportFile>,
    pub file_coverages: Vec<FileCoverage>,
    pub to: Option<PathBuf>,
}

impl CoverageExport {
    pub fn export_to(&mut self, directory: Option<PathBuf>) -> Result<()> {
        self.to = Some(directory.unwrap_or_else(|| PathBuf::from("tmp/qlty-coverage")));
        self.export()
    }

    fn export(&self) -> Result<()> {
        let directory = self.to.as_ref().unwrap();

        GzFormatter::new(JsonEachRowFormatter::new(self.report_files.clone()))
            .write_to_file(&directory.join("report_files.json.gz"))?;

        GzFormatter::new(JsonEachRowFormatter::new(self.file_coverages.clone()))
            .write_to_file(&directory.join("file_coverages.json.gz"))?;

        JsonFormatter::new(self.metadata.clone())
            .write_to_file(&directory.join("metadata.json"))?;

        let raw_file_paths = self
            .report_files
            .iter()
            .map(|report_file| &report_file.path)
            .cloned()
            .collect();

        compress_files(raw_file_paths, "raw_files.zip")
    }

    pub fn total_size_bytes(&self) -> Result<u64> {
        let mut bytes: u64 = 0;

        bytes += self.read_file("report_files.json.gz")?.len() as u64;
        bytes += self.read_file("file_coverages.json.gz")?.len() as u64;
        bytes += self.read_file("metadata.json")?.len() as u64;
        bytes += self.read_file("raw_files.zip")?.len() as u64;

        Ok(bytes)
    }

    pub fn read_file<P: AsRef<Path>>(&self, filename: P) -> Result<Vec<u8>> {
        let path = self.to.as_ref().unwrap().join(filename.as_ref());
        let mut file =
            File::open(&path).with_context(|| format!("Failed to open file: {:?}", path))?;

        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)
            .with_context(|| format!("Failed to read file: {:?}", path))?;

        Ok(buffer)
    }
}
