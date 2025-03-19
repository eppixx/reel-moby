pub trait DisplayDurationExt {
    /// displays a duration in a human readable form
    fn display(&self) -> String;
}

impl DisplayDurationExt for chrono::Duration {
    fn display(&self) -> String {
        if self.num_weeks() == 52 {
            format!("{} Year", (self.num_weeks() / 52) as i32)
        } else if self.num_weeks() > 103 {
            format!("{} Years", (self.num_weeks() / 52) as i32)
        } else if self.num_days() == 1 {
            format!("{} Day", self.num_days())
        } else if self.num_days() > 1 {
            format!("{} Days", self.num_days())
        } else if self.num_hours() == 1 {
            format!("{} Hour", self.num_hours())
        } else if self.num_hours() > 1 {
            format!("{} Hours", self.num_hours())
        } else if self.num_minutes() == 1 {
            format!("{} Minute", self.num_minutes())
        } else if self.num_minutes() > 1 {
            format!("{} Minutes", self.num_minutes())
        } else {
            format!("{} Seconds", self.num_seconds())
        }
    }
}
