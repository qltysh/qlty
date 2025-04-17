use crate::parser;
use crate::Parser;
use anyhow::{bail, Result};
use core::str;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::path::Path;
use std::str::FromStr;

#[derive(clap::ValueEnum, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Formats {
    Simplecov,
    Clover,
    Cobertura,
    Coverprofile,
    Lcov,
    Jacoco,
    Qlty,
}

impl std::fmt::Display for Formats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Formats::Simplecov => write!(f, "simplecov"),
            Formats::Clover => write!(f, "clover"),
            Formats::Cobertura => write!(f, "cobertura"),
            Formats::Coverprofile => write!(f, "coverprofile"),
            Formats::Lcov => write!(f, "lcov"),
            Formats::Jacoco => write!(f, "jacoco"),
            Formats::Qlty => write!(f, "qlty"),
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
                } else {
                    Ok(Formats::Cobertura)
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
            "lcov" => Ok(Formats::Lcov),
            "jacoco" => Ok(Formats::Jacoco),
            "qlty" => Ok(Formats::Qlty),
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
        Formats::Lcov => Box::new(parser::Lcov::new()),
        Formats::Jacoco => Box::new(parser::Jacoco::new()),
        Formats::Qlty => Box::new(parser::Qlty::new()),
    }
}
