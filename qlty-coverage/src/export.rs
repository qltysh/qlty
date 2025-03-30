use anyhow::{Context, Result};
use qlty_formats::{Formatter, JsonEachRowFormatter, JsonFormatter};
use qlty_types::tests::v1::{CoverageMetadata, FileCoverage, ReportFile};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::{write::FileOptions, ZipWriter};

fn compress_files(files: HashMap<String, PathBuf>, output_file: &Path) -> Result<()> {
    // Create the output ZIP file
    let zip_file = File::create(output_file)?;
    let mut zip = ZipWriter::new(zip_file);

    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated) // Compression method
        .unix_permissions(0o755);

    for (name, file_path) in &files {
        if file_path.is_file() {
            // Add the file to the archive
            zip.start_file(name, options)?;

            // Write the file content to the archive
            let mut file = File::open(file_path)?;
            std::io::copy(&mut file, &mut zip)?;
        } else {
            eprintln!("Skipping non-file: {}", file_path.to_string_lossy());
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

        let zip_file_contents = self.compute_zip_file_contents(directory)?;

        compress_files(zip_file_contents, &directory.join("coverage.zip"))
    }

    pub fn total_size_bytes(&self) -> Result<u64> {
        Ok(self.read_file("coverage.zip")?.len() as u64)
    }

    fn compute_zip_file_contents(&self, directory: &Path) -> Result<HashMap<String, PathBuf>> {
        let mut files_to_zip = HashMap::new();

        files_to_zip.insert(
            "report_files.jsonl".to_string(),
            directory.join("report_files.jsonl"),
        );
        files_to_zip.insert(
            "file_coverages.jsonl".to_string(),
            directory.join("file_coverages.jsonl"),
        );
        files_to_zip.insert("metadata.json".to_string(), directory.join("metadata.json"));

        for report_file in &self.report_files {
            let actual_path = PathBuf::from(&report_file.path);
            let zip_file_name = PathBuf::from("raw_files").join(&report_file.path);
            files_to_zip.insert(zip_file_name.to_string_lossy().into_owned(), actual_path);
        }

        Ok(files_to_zip)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, TempDir};
    use zip::read::ZipArchive;

    #[test]
    fn test_export_to() {
        let destination_binding = tempdir().unwrap();
        let destination = destination_binding.path();

        let raw_files_temp_binding = TempDir::new_in(".").unwrap();
        let raw_files_dir = raw_files_temp_binding.path();

        let f1 = &raw_files_dir.join("coverage.lcov");
        let mut file = File::create(f1).unwrap();
        writeln!(file, "D").unwrap();

        let metadata = CoverageMetadata::default();
        let relative_raw_file_path = raw_files_dir
            .file_name()
            .map(|name| {
                Path::new(name)
                    .join("coverage.lcov")
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap_or_default();

        // the ReportFile path is what is supplied by a user as one of the path arguments to `qlty coverage publish path/to/coverage.lcov`
        // that path could be relative or absolute (but probably more typically relative)
        let report_files = vec![ReportFile {
            path: relative_raw_file_path.clone(),
            ..Default::default()
        }];
        let file_coverages = vec![FileCoverage::default()];

        let mut export = CoverageExport {
            metadata,
            report_files,
            file_coverages,
            to: None,
        };

        export.export_to(Some(destination.to_path_buf())).unwrap();

        assert!(destination.join("coverage.zip").exists());

        // Verify the contents of the zip file
        let zip_file = File::open(destination.join("coverage.zip")).unwrap();
        let mut zip = ZipArchive::new(zip_file).unwrap();

        assert!(zip.by_name("report_files.jsonl").is_ok());
        assert!(zip.by_name("file_coverages.jsonl").is_ok());
        assert!(zip.by_name("metadata.json").is_ok());
        let raw_file_path = PathBuf::from("raw_files")
            .join(raw_files_dir.file_name().unwrap())
            .join("coverage.lcov");
        assert!(zip
            .by_name(raw_file_path.to_string_lossy().into_owned().as_str())
            .is_ok());
    }
}
