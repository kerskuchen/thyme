use crate::time::{self, DateTimeHelper, TimeDuration, TimeStamp};

use ct_lib_core::{
    indexmap::IndexMap, path_exists, path_last_modified_time, path_without_filename,
};

use chrono::prelude::*;

use std::{collections::HashSet, fmt::Write};

pub const ACTIVITY_NAME_NON_SPECIFIC_WORK: &str = "Work (Non-specific)";
pub const ACTIVITY_NAME_LEAVE: &str = "Leave";
pub const ACTIVITY_NAME_BREAK: &str = "Break";

const DATE_FORMAT_DATABASE: &str = "%Y_%m_%d__%b_%A";
const DATE_FORMAT_TIMESHEET: &str = "Timesheet for %Y-%m-%d";

pub fn write_durations_summary(day_entry: &DayEntry) -> String {
    let mut result = String::new();

    let work_duration_total = day_entry.get_work_duration_total();
    let work_duration_activities = day_entry.get_work_duration_specific();
    let work_duration_non_specific = day_entry.get_work_duration_non_specific();
    let work_percent_specific = (100.0
        * (work_duration_activities.minutes as f32 / work_duration_total.minutes as f32))
        .round() as usize;
    let work_percent_non_specific = 100 - work_percent_specific;
    writeln!(
        result,
        "Total work duration:            {} (100%)",
        work_duration_total.to_string(),
    )
    .unwrap();
    writeln!(
        result,
        "  - Activities (from list):     {} ({: >3}%)",
        work_duration_activities.to_string(),
        work_percent_specific
    )
    .unwrap();
    writeln!(
        result,
        "  - Activities (non-specific):  {} ({: >3}%)",
        work_duration_non_specific.to_string(),
        work_percent_non_specific
    )
    .unwrap();
    writeln!(
        result,
        "Total break duration:           {}",
        day_entry.get_break_duration().to_string(),
    )
    .unwrap();
    if let Some(leave_duration) = day_entry.get_leave_duration() {
        writeln!(
            result,
            "Time since last leave:          {}",
            leave_duration.to_string(),
        )
        .unwrap();
    } else {
        writeln!(result, "").unwrap();
    }

    result
}
pub struct DayEntry {
    pub date: NaiveDate,
    pub activities: Vec<Activity>,
    pub last_write_time: f64,
}

impl DayEntry {
    pub fn create_empty() -> DayEntry {
        let mut result = DayEntry {
            activities: vec![],
            date: time::get_current_date(),
            last_write_time: 0.0,
        };
        result.write_back();
        result
    }

    pub fn load_or_create() -> DayEntry {
        let today_date = time::get_current_date();
        let today_entry = DayEntry {
            activities: vec![Activity {
                is_work: true,
                name: ACTIVITY_NAME_NON_SPECIFIC_WORK.to_owned(),
                time_start: time::get_current_time().to_timestamp(),
                time_end: None,
            }],
            date: today_date,
            last_write_time: 0.0,
        };

        let timesheet_filepath = DayEntry::timesheet_filepath_default();
        let mut result = if path_exists(&timesheet_filepath) {
            let entry = DayEntry::load_from_file(&timesheet_filepath);
            if entry.date == today_date {
                entry
            } else {
                today_entry
            }
        } else {
            today_entry
        };

        result.write_back();
        result
    }

    pub fn hotreload_external_changes(&mut self) {
        let timesheet_filepath = DayEntry::timesheet_filepath_default();
        assert!(path_exists(&timesheet_filepath));

        let last_modified_time = path_last_modified_time(&timesheet_filepath);
        if self.last_write_time < last_modified_time {
            *self = DayEntry::load_from_file(&timesheet_filepath);
            self.write_back();
        }
    }

    fn load_from_file(filepath: &str) -> DayEntry {
        let content = std::fs::read_to_string(&filepath)
            .unwrap_or_else(|error| panic!("Could not read '{}' - {}", &filepath, error));
        let mut lines: Vec<String> = content
            .lines()
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with("---"))
            .map(|line| line.to_owned())
            .collect();
        assert!(!lines.is_empty(), "Found empty timesheet at '{}'", filepath);

        let first_line = lines.remove(0);
        let date =
            NaiveDate::parse_from_str(&first_line, DATE_FORMAT_TIMESHEET).unwrap_or_else(|err| {
                panic!(
                    "First line of timesheet in '{}' is not a valid date: {}",
                    filepath, err
                )
            });

        let stamp_events: Vec<StampEvent> = lines
            .into_iter()
            .map(|line| StampEvent::from_string(&line))
            .collect();

        // Check if stamps are in correct order
        stamp_events.iter().fold(
            TimeStamp {
                hours: 0,
                minutes: 0,
            },
            |previous, event| {
                let timestamp = event.timestamp();
                assert!(
                    previous < timestamp,
                    "Found stamp event '{}' that begins earlier than previous event in list at {}",
                    event.to_string(),
                    previous.to_string()
                );
                timestamp
            },
        );

        let activities = DayEntry::create_activities_from_stamp_events(&stamp_events);
        DayEntry {
            date,
            activities,
            last_write_time: 0.0,
        }
    }

    pub fn write_back(&mut self) {
        let database_directory_path =
            path_without_filename(&DayEntry::timesheet_filepath_for_date(self.date));
        if !path_exists(&database_directory_path) {
            std::fs::create_dir_all(&database_directory_path).unwrap_or_else(|error| {
                panic!(
                    "Could not crate path '{}' - {}",
                    &database_directory_path, error
                )
            });
        }
        self.write_timesheets();
        self.write_report();
    }

    pub fn write_timesheets(&mut self) {
        let timesheet = self.generate_timesheet();

        let filepath_default = DayEntry::timesheet_filepath_default();
        std::fs::write(&filepath_default, &timesheet).unwrap_or_else(|error| {
            panic!("Could not write to '{}' - {}", &filepath_default, error)
        });
        let filepath_database = DayEntry::timesheet_filepath_for_date(self.date);
        std::fs::write(&filepath_database, &timesheet).unwrap_or_else(|error| {
            panic!("Could not write to '{}' - {}", &filepath_database, error)
        });

        self.last_write_time = path_last_modified_time(&filepath_default);
    }

    fn generate_timesheet(&self) -> String {
        let mut result = String::new();

        writeln!(result, "{}", self.date.format(DATE_FORMAT_TIMESHEET)).unwrap();
        writeln!(result, "------------------------\n").unwrap();
        let stamp_events = DayEntry::create_stamp_events_from_activities(&self.activities);
        for stamp_event in &stamp_events {
            writeln!(result, "{}", stamp_event.to_string()).unwrap();
        }
        result
    }

    pub fn write_report(&self) {
        let report = self.generate_report();

        let report_filepath = DayEntry::report_filepath_for_date(self.date);
        std::fs::write(&report_filepath, &report).unwrap_or_else(|error| {
            panic!("Could not write to '{}' - {}", &report_filepath, error)
        });
        let report_filepath_default = DayEntry::report_filepath_default();
        std::fs::write(&report_filepath_default, &report).unwrap_or_else(|error| {
            panic!("Could not write to '{}' - {}", &report_filepath, error)
        });
    }

    fn generate_report(&self) -> String {
        let mut result = String::new();
        let checkin_date = self.date;

        writeln!(
            result,
            "Report for {}\n",
            checkin_date.format("%A %e. %b (%d.%m.%Y)"),
        )
        .unwrap();

        writeln!(result, "\nActivity Durations:").unwrap();
        writeln!(result, "=====================\n").unwrap();

        for (activity_name, duration) in self.get_activity_durations().into_iter() {
            writeln!(result, "{} - {}", duration.to_string(), activity_name).unwrap();
        }

        // Totals summary
        writeln!(result, "\n-------------\n").unwrap();

        writeln!(result, "{}", &write_durations_summary(&self)).unwrap();

        // Activity list
        writeln!(result, "\nDetailed Activity List:").unwrap();
        writeln!(result, "=========================\n").unwrap();
        for activity in self.activities.iter() {
            writeln!(result, "{}", activity.to_string()).unwrap();
        }
        writeln!(result, "").unwrap();

        result
    }

    pub fn start_activitiy(&mut self, name: &str, is_work: bool) {
        let timestamp_now = time::get_current_datetime().to_timestamp();

        // Close previous activity
        if let Some(current) = self.get_current_activity_mut() {
            if current.name == name {
                debug_assert!(false, "Trying to start same activity '{}' twice", name);
                return;
            }
            assert!(current.time_end.is_none());
            current.time_end = Some(timestamp_now);
        }

        // Start new activity
        self.activities.push(Activity {
            is_work,
            name: name.to_owned(),
            time_start: timestamp_now,
            time_end: None,
        });

        DayEntry::cleanup_activities(&mut self.activities);
        self.write_back();
    }

    pub fn is_currently_working(&self) -> bool {
        if let Some(activity) = self.get_current_activity() {
            activity.is_work
        } else {
            false
        }
    }

    pub fn get_current_activity(&self) -> Option<&Activity> {
        self.activities.last()
    }

    fn get_current_activity_mut(&mut self) -> Option<&mut Activity> {
        self.activities.last_mut()
    }

    pub fn first_checkin_time(&self) -> Option<TimeStamp> {
        self.activities.first().map(|activity| activity.time_start)
    }

    pub fn get_work_duration_total(&self) -> TimeDuration {
        self.activities
            .iter()
            .filter(|activity| activity.is_work)
            .fold(TimeDuration::zero(), |acc, activity| {
                acc + activity.duration()
            })
    }

    pub fn get_work_duration_specific(&self) -> TimeDuration {
        self.activities
            .iter()
            .filter(|activity| activity.is_work)
            .filter(|activity| activity.name != ACTIVITY_NAME_NON_SPECIFIC_WORK)
            .fold(TimeDuration::zero(), |acc, activity| {
                acc + activity.duration()
            })
    }

    pub fn get_work_duration_non_specific(&self) -> TimeDuration {
        self.activities
            .iter()
            .filter(|activity| activity.is_work)
            .filter(|activity| activity.name == ACTIVITY_NAME_NON_SPECIFIC_WORK)
            .fold(TimeDuration::zero(), |acc, activity| {
                acc + activity.duration()
            })
    }

    pub fn get_break_duration(&self) -> TimeDuration {
        self.activities
            .iter()
            .filter(|activity| !activity.is_work)
            .filter(|activity| activity.time_end.is_some())
            .fold(TimeDuration::zero(), |acc, activity| {
                acc + activity.duration()
            })
    }

    pub fn get_leave_duration(&self) -> Option<TimeDuration> {
        if self.get_current_activity().is_none() {
            return None;
        }
        if self.is_currently_working() {
            None
        } else {
            Some(
                self.activities
                    .iter()
                    .filter(|activity| !activity.is_work)
                    .filter(|activity| activity.time_end.is_none())
                    .fold(TimeDuration::zero(), |acc, activity| {
                        acc + activity.duration()
                    }),
            )
        }
    }

    pub fn get_non_work_duration(&self) -> TimeDuration {
        self.activities
            .iter()
            .filter(|activity| !activity.is_work)
            .fold(TimeDuration::zero(), |acc, activity| {
                acc + activity.duration()
            })
    }

    pub fn get_activity_durations(&self) -> IndexMap<String, TimeDuration> {
        let activity_names: HashSet<String> = self
            .activities
            .iter()
            .filter(|activity| activity.is_work)
            .map(|activity| activity.name.clone())
            .collect();

        let mut activity_names_and_durations: IndexMap<String, TimeDuration> = activity_names
            .into_iter()
            .map(|activity_name| {
                let duration = self
                    .activities
                    .iter()
                    .filter(|activity| activity.name == activity_name)
                    .fold(TimeDuration::zero(), |acc, activity| {
                        acc + activity.duration()
                    });

                (activity_name, duration)
            })
            .collect();

        activity_names_and_durations.sort_by(
            |_activity_name_a, duration_a, _activity_name_b, duration_b| {
                // NOTE: The negatives forces descending sorting
                (-duration_a.minutes).cmp(&-duration_b.minutes)
            },
        );

        activity_names_and_durations
    }

    fn create_activities_from_stamp_events(stamp_events: &[StampEvent]) -> Vec<Activity> {
        let mut result = Vec::new();
        let mut current_activity: Option<Activity> = None;
        for event in stamp_events.iter() {
            match event {
                StampEvent::Begin(timestamp, activity_name) => {
                    // Close current activity
                    if current_activity.is_some() {
                        if current_activity.as_ref().unwrap().name == *activity_name {
                            panic!(
                                "Got a duplicate activity '{}' at {}",
                                activity_name,
                                timestamp.to_string()
                            )
                        }
                        current_activity.as_mut().unwrap().time_end = Some(*timestamp);
                        result.push(current_activity.take().unwrap());
                    }

                    // Start new activity
                    current_activity = Some(Activity {
                        is_work: true,
                        name: activity_name.to_owned(),
                        time_start: *timestamp,
                        time_end: None,
                    });
                }
                StampEvent::Leave(timestamp) => {
                    // Close current activity
                    if current_activity.is_some() {
                        if !current_activity.as_ref().unwrap().is_work {
                            panic!(
                                "Got a duplicate leave activity at {}",
                                timestamp.to_string()
                            )
                        }
                        current_activity.as_mut().unwrap().time_end = Some(*timestamp);
                        result.push(current_activity.take().unwrap());
                    }

                    // Start new activity
                    current_activity = Some(Activity {
                        is_work: false,
                        name: ACTIVITY_NAME_LEAVE.to_owned(),
                        time_start: *timestamp,
                        time_end: None,
                    });
                }
            }
        }
        if let Some(current_activity) = current_activity {
            result.push(current_activity);
        }

        DayEntry::cleanup_activities(&mut result);
        result
    }

    fn create_stamp_events_from_activities(activities: &[Activity]) -> Vec<StampEvent> {
        let mut result = Vec::new();
        for activity in activities.iter() {
            if activity.is_work {
                result.push(StampEvent::Begin(
                    activity.time_start,
                    activity.name.clone(),
                ));
            } else {
                result.push(StampEvent::Leave(activity.time_start));
            }
        }
        result
    }

    fn cleanup_activities(activities: &mut Vec<Activity>) {
        // let mut debug = String::new();
        // writeln!(debug, "before");
        // for a in activities.iter() {
        //     writeln!(debug, "{}", a.to_string());
        // }

        // Remove zero sized activities
        activities
            .retain(|activity| activity.time_end.is_none() || activity.duration().minutes != 0);

        // writeln!(debug, "after remove zeroes");
        // for a in activities.iter() {
        //     writeln!(debug, "{}", a.to_string());
        // }
        // std::fs::write("debug.txt", &debug);

        // Rename all "leave" activities to "leave" so the next merge operation is easier
        for activity in activities.iter_mut() {
            if activity.is_work {
                continue;
            }
            activity.name = ACTIVITY_NAME_LEAVE.to_owned();
        }

        // writeln!(debug, "after rename");
        // for a in activities.iter() {
        //     writeln!(debug, "{}", a.to_string());
        // }
        // std::fs::write("debug.txt", &debug);

        // Merge adjacent same activities
        // NOTE: These can occur after the previous removal operation
        let mut result = Vec::new();
        let mut current_activity: Option<Activity> = None;
        for activity in activities.drain(..) {
            if current_activity.is_none() {
                current_activity = Some(activity);
                continue;
            }

            // Merge if necessary
            if let Some(current_activity) = current_activity.as_mut() {
                if current_activity.is_work == activity.is_work
                    && current_activity.name == activity.name
                {
                    current_activity.time_end = activity.time_end;
                    continue;
                }
            }

            result.push(current_activity.take().unwrap());
            current_activity = Some(activity);
        }
        if let Some(current_activity) = current_activity {
            result.push(current_activity);
        }

        *activities = result;

        // writeln!(debug, "after merge");
        // for a in activities.iter() {
        //     writeln!(debug, "{}", a.to_string());
        // }
        // std::fs::write("debug.txt", &debug);

        // Rename "leave" activities to "break" is the have a confirmed end
        for activity in activities.iter_mut() {
            if activity.is_work {
                continue;
            }
            if activity.time_end.is_some() {
                activity.name = ACTIVITY_NAME_BREAK.to_owned();
            }
        }

        // writeln!(debug, "after rename");
        // for a in activities.iter() {
        //     writeln!(debug, "{}", a.to_string());
        // }
        // std::fs::write("debug.txt", &debug);
    }

    fn timesheet_filepath_for_date(date: NaiveDate) -> String {
        format!(
            "database/{}__timesheet.txt",
            date.format(DATE_FORMAT_DATABASE)
        )
    }
    fn timesheet_filepath_default() -> String {
        "today__timesheet.txt".to_owned()
    }
    fn report_filepath_for_date(date: NaiveDate) -> String {
        format!("database/{}__report.txt", date.format(DATE_FORMAT_DATABASE))
    }
    fn report_filepath_default() -> String {
        "today__report.txt".to_owned()
    }
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub is_work: bool,
    pub name: String,
    pub time_start: TimeStamp,
    pub time_end: Option<TimeStamp>,
}

impl Activity {
    pub fn to_string(&self) -> String {
        let time_range = if let Some(end) = self.time_end {
            format!("{} - {}", self.time_start.to_string(), end.to_string())
        } else {
            format!("{} - {}", self.time_start.to_string(), "<now>")
        };

        format!(
            "{} [{}] - [{}]",
            time_range,
            self.duration().to_string(),
            self.name,
        )
    }

    pub fn duration(&self) -> TimeDuration {
        if let Some(end) = self.time_end {
            end - self.time_start
        } else {
            time::get_current_datetime().to_timestamp() - self.time_start
        }
    }
}

#[derive(Debug, Clone)]
enum StampEvent {
    Begin(TimeStamp, String),
    Leave(TimeStamp),
}

impl StampEvent {
    fn timestamp(&self) -> TimeStamp {
        match self {
            StampEvent::Begin(timestamp, _name) => timestamp.clone(),
            StampEvent::Leave(timestamp) => timestamp.clone(),
        }
    }

    fn to_string(&self) -> String {
        match self {
            StampEvent::Begin(timestamp, name) => {
                format!("{} - Begin [{}]", timestamp.to_string(), name)
            }
            StampEvent::Leave(timestamp) => format!("{} - Leave", timestamp.to_string()),
        }
    }

    fn from_string(input: &str) -> StampEvent {
        let re_begin = regex::Regex::new(r"(\d{2}:\d{2}) - Begin (\[.+\])").unwrap();
        for capture in re_begin.captures_iter(input) {
            let timestamp = TimeStamp::from_string(&capture[1]);
            let activity_name = capture[2]
                .strip_prefix("[")
                .unwrap()
                .strip_suffix("]")
                .unwrap()
                .to_owned();
            return StampEvent::Begin(timestamp, activity_name);
        }

        let re_leave = regex::Regex::new(r"(\d{2}:\d{2}) - Leave").unwrap();
        for capture in re_leave.captures_iter(input) {
            let timestamp = TimeStamp::from_string(&capture[1]);
            return StampEvent::Leave(timestamp);
        }

        panic!("The string '{}' is not a valid stamp event", input)
    }
}
