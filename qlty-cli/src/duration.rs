use anyhow::{anyhow, Result};
use std::time::Duration;

/// Parse a duration string like "5m", "300s", "1h" into a Duration
pub fn parse_duration(s: &str) -> Result<Duration> {
    if s.is_empty() {
        return Err(anyhow!("Empty duration string"));
    }

    let (num_part, unit_part) = split_duration_string(s)?;
    let value: f64 = num_part
        .parse()
        .map_err(|_| anyhow!("Invalid number in duration: {}", num_part))?;

    if value < 0.0 {
        return Err(anyhow!("Duration cannot be negative"));
    }

    let seconds = match unit_part {
        "s" | "sec" | "secs" | "second" | "seconds" => value,
        "m" | "min" | "mins" | "minute" | "minutes" => value * 60.0,
        "h" | "hr" | "hrs" | "hour" | "hours" => value * 3600.0,
        "" => value, // Default to seconds if no unit specified
        _ => return Err(anyhow!("Unknown duration unit: {}", unit_part)),
    };

    Ok(Duration::from_secs_f64(seconds))
}

fn split_duration_string(s: &str) -> Result<(&str, &str)> {
    // Find the position where the numeric part ends
    let split_pos = s.chars().position(|c| c.is_alphabetic()).unwrap_or(s.len());

    let (num_part, unit_part) = s.split_at(split_pos);

    if num_part.is_empty() {
        return Err(anyhow!("Missing numeric value in duration"));
    }

    Ok((num_part, unit_part))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_seconds() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("30sec").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("30secs").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("30second").unwrap(), Duration::from_secs(30));
        assert_eq!(
            parse_duration("30seconds").unwrap(),
            Duration::from_secs(30)
        );
        assert_eq!(parse_duration("30").unwrap(), Duration::from_secs(30));
    }

    #[test]
    fn test_parse_minutes() {
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("5min").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("5mins").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("5minute").unwrap(), Duration::from_secs(300));
        assert_eq!(
            parse_duration("5minutes").unwrap(),
            Duration::from_secs(300)
        );
        assert_eq!(parse_duration("1.5m").unwrap(), Duration::from_secs(90));
    }

    #[test]
    fn test_parse_hours() {
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("2hr").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("2hrs").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("2hour").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("2hours").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("0.5h").unwrap(), Duration::from_secs(1800));
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("-5s").is_err());
        assert!(parse_duration("5x").is_err());
        assert!(parse_duration("m").is_err());
    }
}
