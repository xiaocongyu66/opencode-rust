use chrono::{DateTime, Local, Utc};

pub fn titlecase(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_is_boundary = true;
    for ch in s.chars() {
        if prev_is_boundary && ch.is_alphanumeric() {
            result.extend(ch.to_uppercase());
        } else {
            result.push(ch);
        }
        prev_is_boundary = !ch.is_alphanumeric();
    }
    result
}

pub fn time(input: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp_millis(input)
        .unwrap_or_default()
        .with_timezone(&Local);
    dt.format("%H:%M").to_string()
}

pub fn datetime(input: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp_millis(input)
        .unwrap_or_default()
        .with_timezone(&Local);
    format!("{} · {}", dt.format("%H:%M"), dt.format("%x"))
}

pub fn today_time_or_datetime(input: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp_millis(input)
        .unwrap_or_default()
        .with_timezone(&Local);
    let now = Local::now();
    let is_today = dt.date_naive() == now.date_naive();
    if is_today {
        time(input)
    } else {
        datetime(input)
    }
}

pub fn format_number(num: u64) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1000 {
        format!("{:.1}K", num as f64 / 1000.0)
    } else {
        num.to_string()
    }
}

pub fn duration(input: u64) -> String {
    if input < 1000 {
        return format!("{}ms", input);
    }
    if input < 60000 {
        return format!("{:.1}s", input as f64 / 1000.0);
    }
    if input < 3_600_000 {
        let minutes = input / 60000;
        let seconds = (input % 60000) / 1000;
        return format!("{}m {}s", minutes, seconds);
    }
    if input < 86_400_000 {
        let hours = input / 3_600_000;
        let minutes = (input % 3_600_000) / 60000;
        return format!("{}h {}m", hours, minutes);
    }
    let days = input / 86_400_000;
    let hours = (input % 86_400_000) / 3_600_000;
    format!("{}d {}h", days, hours)
}

pub fn truncate(s: &str, len: usize) -> String {
    if s.chars().count() <= len {
        return s.to_string();
    }
    let take = len.saturating_sub(1);
    let truncated: String = s.chars().take(take).collect();
    format!("{}…", truncated)
}

pub fn truncate_left(s: &str, len: usize) -> String {
    let count = s.chars().count();
    if count <= len {
        return s.to_string();
    }
    let take = len.saturating_sub(1);
    let skipped = count.saturating_sub(take);
    let result: String = s.chars().skip(skipped).collect();
    format!("…{}", result)
}

pub fn truncate_middle(s: &str, max_len: usize) -> String {
    let count = s.chars().count();
    if count <= max_len {
        return s.to_string();
    }
    let ellipsis = "…";
    let ellipsis_len = ellipsis.chars().count();
    let keep_total = max_len.saturating_sub(ellipsis_len);
    let keep_start = (keep_total + 1) / 2;
    let keep_end = keep_total / 2;
    let start: String = s.chars().take(keep_start).collect();
    let end: String = s.chars().skip(count.saturating_sub(keep_end)).collect();
    format!("{}{}{}", start, ellipsis, end)
}

pub fn pluralize(count: u64, singular: &str, plural: &str) -> String {
    let template = if count == 1 { singular } else { plural };
    template.replace("{}", &count.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titlecase() {
        assert_eq!(titlecase("hello world"), "Hello World");
        assert_eq!(titlecase("foo-bar-baz"), "Foo-Bar-Baz");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1500), "1.5K");
        assert_eq!(format_number(2_500_000), "2.5M");
    }

    #[test]
    fn test_duration() {
        assert_eq!(duration(500), "500ms");
        assert_eq!(duration(1500), "1.5s");
        assert_eq!(duration(65000), "1m 5s");
        assert_eq!(duration(3_700_000), "1h 0m");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hell…");
    }

    #[test]
    fn test_truncate_left() {
        assert_eq!(truncate_left("hello", 10), "hello");
        assert_eq!(truncate_left("hello world", 5), "…orld");
    }

    #[test]
    fn test_truncate_middle() {
        let result = truncate_middle("abcdefghij", 5);
        assert_eq!(result, "ab…ij");
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize(1, "{} item", "{} items"), "1 item");
        assert_eq!(pluralize(5, "{} item", "{} items"), "5 items");
    }
}
