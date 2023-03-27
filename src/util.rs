use chrono::Duration;

pub fn duration_to_hms(dur: &Duration) -> (i64, i64, i64) {
    let hours = dur.num_hours() % 100;
    let minutes = dur.num_minutes() % 60;
    let seconds = dur.num_seconds() % 60;

    (hours, minutes, seconds)
}

pub fn hms_to_timer_string(h: i64, m: i64, s:i64) -> String {
    format!("{:02}:{:02}:{:02}", h, m, s)
}

pub fn duration_to_timer_string(dur: &Duration) -> String {
    let (h, m, s) = duration_to_hms(dur);
    hms_to_timer_string(h, m, s)
}
