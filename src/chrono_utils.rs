use chrono::Duration;

pub fn render_hours_mins(t: Duration) -> String {
    format!("{}:{:02}", t.num_hours(), t.num_minutes() % 60)
}
