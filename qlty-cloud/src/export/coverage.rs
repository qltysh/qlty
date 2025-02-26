use crate::format::{JsonEachRowFormatter, JsonFormatter};
use anyhow::{Context, Result};
use qlty_types::tests::v1::{CoverageMetadata, FileCoverage, ReportFile};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::{write::FileOptions, ZipWriter};

fn compress_files(
    files: Vec<String>,
    output_file: &Path,
    strip_prefix: Option<&Path>,
) -> Result<()> {
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
            // Determine the filename to use in the ZIP file
            let filename = if let Some(prefix) = strip_prefix {
                path.strip_prefix(prefix)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .into_owned()
            } else {
                path.to_string_lossy().into_owned()
            };

            // Add the file to the archive
            zip.start_file(filename, options)?;

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

        JsonEachRowFormatter::new(self.report_files.clone())
            .write_to_file(&directory.join("report_files.jsonl"))?;

        JsonEachRowFormatter::new(self.file_coverages.clone())
            .write_to_file(&directory.join("file_coverages.jsonl"))?;

        JsonFormatter::new(self.metadata.clone())
            .write_to_file(&directory.join("metadata.json"))?;

        let raw_files_dir = directory.join("raw_files");
        let exported_raw_files = self.export_raw_report_files(&raw_files_dir)?;

        let mut files_to_zip = [
            "report_files.jsonl",
            "file_coverages.jsonl",
            "metadata.json",
        ]
        .iter()
        .map(|file| directory.join(file).to_string_lossy().into_owned())
        .collect::<Vec<_>>();

        files_to_zip.extend(exported_raw_files);

        compress_files(
            files_to_zip,
            &directory.join("coverage.zip"),
            Some(directory),
        )
    }

    pub fn total_size_bytes(&self) -> Result<u64> {
        Ok(self.read_file("coverage.zip")?.len() as u64)
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

    fn export_raw_report_files(&self, output_dir: &Path) -> Result<Vec<String>> {
        let mut copied_files = Vec::new();

        for report_file in &self.report_files {
            let path = Path::new(&report_file.path);

            if path.is_file() {
                let relative_path = path.strip_prefix("/").unwrap_or(path);
                let dest_path = output_dir.join(relative_path);
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(path, dest_path.as_path())?;
                copied_files.push(dest_path.to_string_lossy().into_owned());
            } else {
                eprintln!("Skipping non-file: {}", report_file.path);
            }
        }

        Ok(copied_files)
    }
}
