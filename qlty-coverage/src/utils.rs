use crate::formats::Formats;
use anyhow::Result;
use std::path::PathBuf;
use std::str::FromStr;

// Format specification priority:
// 1. The format specified in the path, e.g.: simplecov:./coverage/coverage.json
// 2. The format specified in the command line arguments, through --report-format simplecov
// 3. The format is inferred from the file extension or contents
// TODO: Write unit tests
pub fn extract_path_and_format(
    path: &str,
    base_format: Option<Formats>,
) -> Result<(PathBuf, Formats)> {
    if path.contains(':') {
        match path.split_once(':') {
            Some((format, p)) => Ok((PathBuf::from(p), Formats::from_str(format)?)),
            None => Err(anyhow::anyhow!(
                "Expected ':' in the path to split format and path."
            )),
        }
    } else if base_format.is_some() {
        return Ok((PathBuf::from(path), base_format.unwrap()));
    } else {
        let format = Formats::try_from(path.as_ref())?;
        return Ok((PathBuf::from(path), format));
    }
}
