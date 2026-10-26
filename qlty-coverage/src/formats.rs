use crate::parser;
use crate::Parser;
use anyhow::{bail, Result};
use core::str;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::path::Path;
use std::str::FromStr;

#[derive(clap::ValueEnum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Formats {
    Simplecov,
    Clover,
    Cobertura,
    Coverprofile,
    Dotcover,
    Lcov,
    Jacoco,
    Qlty,
    XccovJson,
}

impl std::fmt::Display for Formats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Formats::Simplecov => write!(f, "simplecov"),
            Formats::Clover => write!(f, "clover"),
            Formats::Cobertura => write!(f, "cobertura"),
            Formats::Coverprofile => write!(f, "coverprofile"),
            Formats::Dotcover => write!(f, "dotcover"),
            Formats::Lcov => write!(f, "lcov"),
            Formats::Jacoco => write!(f, "jacoco"),
            Formats::Qlty => write!(f, "qlty"),
            Formats::XccovJson => write!(f, "xccov-json"),
        }
    }
}

impl TryFrom<&Path> for Formats {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self> {
        match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("info") | Some("lcov") => Ok(Formats::Lcov),
            Some("json") => Ok(Formats::Simplecov),
            Some("jsonl") => Ok(Formats::Qlty),
            Some("out") => Ok(Formats::Coverprofile),
            Some("xml") => {
                let path_str = path.to_str().unwrap();
                if path_str.contains("jacoco") {
                    Ok(Formats::Jacoco)
                } else if path_str.contains("clover") {
                    Ok(Formats::Clover)
                } else if path_str.contains("dotcover") {
                    Ok(Formats::Dotcover)
                } else {
                    // Try to detect dotCover by reading content
                    match std::fs::read_to_string(path) {
                        Ok(content) if content.contains("DotCoverVersion") => Ok(Formats::Dotcover),
                        _ => Ok(Formats::Cobertura),
                    }
                }
            }
            Some(other) => bail!("Unknown file extension for coverage report: {}\nSpecify the format with --report-format=FORMAT", other),
            None => bail!(
                "Could not determine a report format by file extension: {}\nSpecify the format with --report-format=FORMAT",
                path.display()
            ),
        }
    }
}

impl FromStr for Formats {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "simplecov" => Ok(Formats::Simplecov),
            "clover" => Ok(Formats::Clover),
            "cobertura" => Ok(Formats::Cobertura),
            "coverprofile" => Ok(Formats::Coverprofile),
            "dotcover" => Ok(Formats::Dotcover),
            "lcov" => Ok(Formats::Lcov),
            "jacoco" => Ok(Formats::Jacoco),
            "qlty" => Ok(Formats::Qlty),
            "xccov-json" => Ok(Formats::XccovJson),
            _ => bail!("Unsupported coverage report format: {}", s),
        }
    }
}

pub fn parser_for(&format: &Formats) -> Box<dyn Parser> {
    match format {
        Formats::Simplecov => Box::new(parser::Simplecov::new()),
        Formats::Clover => Box::new(parser::Clover::new()),
        Formats::Cobertura => Box::new(parser::Cobertura::new()),
        Formats::Coverprofile => Box::new(parser::Coverprofile::new()),
        Formats::Dotcover => Box::new(parser::Dotcover::new()),
        Formats::Lcov => Box::new(parser::Lcov::new()),
        Formats::Jacoco => Box::new(parser::Jacoco::new()),
        Formats::Qlty => Box::new(parser::Qlty::new()),
        Formats::XccovJson => Box::new(parser::XccovJson::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_lcov_extension() {
        let path = Path::new("coverage.lcov");
        let format = Formats::try_from(path).unwrap();
        assert_eq!(format, Formats::Lcov);
    }

    #[test]
    fn test_try_from_info_extension() {
        let path = Path::new("coverage.info");
        let format = Formats::try_from(path).unwrap();
        assert_eq!(format, Formats::Lcov);
    }

    #[test]
    fn test_try_from_json_extension() {
        let path = Path::new("coverage.json");
        let format = Formats::try_from(path).unwrap();
        assert_eq!(format, Formats::Simplecov);
    }

    #[test]
    fn test_try_from_jsonl_extension() {
        let path = Path::new("coverage.jsonl");
        let format = Formats::try_from(path).unwrap();
        assert_eq!(format, Formats::Qlty);
    }

    #[test]
    fn test_try_from_out_extension() {
        let path = Path::new("coverage.out");
        let format = Formats::try_from(path).unwrap();
        assert_eq!(format, Formats::Coverprofile);
    }

    #[test]
    fn test_try_from_jacoco_xml() {
        let path = Path::new("jacoco.xml");
        let format = Formats::try_from(path).unwrap();
        assert_eq!(format, Formats::Jacoco);
    }

    #[test]
    fn test_try_from_clover_xml() {
        let path = Path::new("clover.xml");
        let format = Formats::try_from(path).unwrap();
        assert_eq!(format, Formats::Clover);
    }

    #[test]
    fn test_try_from_dotcover_xml() {
        let path = Path::new("dotcover.xml");
        let format = Formats::try_from(path).unwrap();
        assert_eq!(format, Formats::Dotcover);
    }

    #[test]
    fn test_try_from_cobertura_xml() {
        let path = Path::new("coverage.xml");
        let format = Formats::try_from(path).unwrap();
        assert_eq!(format, Formats::Cobertura);
    }

    #[test]
    fn test_try_from_dotcover_xml_by_content() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test.xml");
        std::fs::write(&path, "<coverage DotCoverVersion=\"1.0\"></coverage>").unwrap();
        let format = Formats::try_from(path.as_path()).unwrap();
        assert_eq!(format, Formats::Dotcover);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_try_from_unknown_extension() {
        let path = Path::new("coverage.txt");
        let result = Formats::try_from(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_no_extension() {
        let path = Path::new("coverage");
        let result = Formats::try_from(path);
        assert!(result.is_err());
    }
}
