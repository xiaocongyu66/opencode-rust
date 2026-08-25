pub fn format_duration(secs: u64) -> String {
    if secs == 0 {
        return String::new();
    }
    if secs < 60 {
        return format!("{}s", secs);
    }
    if secs < 3600 {
        let mins = secs / 60;
        let remaining = secs % 60;
        return if remaining > 0 {
            format!("{}m {}s", mins, remaining)
        } else {
            format!("{}m", mins)
        };
    }
    if secs < 86400 {
        let hours = secs / 3600;
        let remaining = (secs % 3600) / 60;
        return if remaining > 0 {
            format!("{}h {}m", hours, remaining)
        } else {
            format!("{}h", hours)
        };
    }
    if secs < 604800 {
        let days = secs / 86400;
        return if days == 1 {
            "~1 day".to_string()
        } else {
            format!("~{} days", days)
        };
    }
    let weeks = secs / 604800;
    if weeks == 1 {
        "~1 week".to_string()
    } else {
        format!("~{} weeks", weeks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        assert_eq!(format_duration(0), "");
    }

    #[test]
    fn test_seconds() {
        assert_eq!(format_duration(45), "45s");
    }

    #[test]
    fn test_minutes() {
        assert_eq!(format_duration(120), "2m");
        assert_eq!(format_duration(90), "1m 30s");
    }

    #[test]
    fn test_hours() {
        assert_eq!(format_duration(3600), "1h");
        assert_eq!(format_duration(5400), "1h 30m");
    }

    #[test]
    fn test_days() {
        assert_eq!(format_duration(86400), "~1 day");
        assert_eq!(format_duration(172800), "~2 days");
    }

    #[test]
    fn test_weeks() {
        assert_eq!(format_duration(604800), "~1 week");
        assert_eq!(format_duration(1209600), "~2 weeks");
    }
}
