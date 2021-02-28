use chrono::{prelude::*, Local};
use crossterm::{
    cursor::{self},
    event::{KeyCode, KeyEvent, KeyModifiers},
    style::Print,
    terminal::{DisableLineWrap, EnableLineWrap, SetTitle},
    ExecutableCommand,
};

use ct_lib_core::{path_exists, path_without_filename};

use std::{collections::HashSet, fmt::Write};

fn main() -> crossterm::Result<()> {
    let mut day_entry = DayEntry::load_or_create();

    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    stdout.execute(DisableLineWrap)?;

    let mut previous_time = Local::now().to_timestamp();
    let mut is_running = true;
    while is_running {
        let project_names_list = reload_project_names();

        // Write changes every minute
        let current_time = Local::now().to_timestamp();
        if (current_time - previous_time).minutes > 0 {
            // One minute has passed
            day_entry.write_back();
            previous_time = current_time;
        } else if (current_time - previous_time).minutes < 0 {
            // A whole day has passed - we need to create a new entry
            day_entry = DayEntry::create_empty();
            previous_time = current_time;
        }

        let (terminal_width, terminal_height) = crossterm::terminal::size().unwrap_or((100, 30));
        let terminal_width = (terminal_width - 2) as usize;
        let terminal_height = (terminal_height.min(30) - 2) as usize;

        let clear_screen = create_clear_screen(terminal_width, terminal_height);
        let main_screen = create_main_screen(
            &day_entry,
            &project_names_list,
            terminal_width,
            terminal_height,
        );

        stdout
            .execute(cursor::MoveTo(0, 0))?
            //.execute(Clear(ClearType::All))? // This just scrolls down on Windows and looks glitchy
            .execute(Print(&clear_screen))?
            .execute(cursor::MoveTo(0, 0))?
            .execute(Print(&main_screen))?
            .execute(cursor::MoveTo(0, 0))?;

        let title = if let Some(current_activity) = day_entry.get_current_activity() {
            format!(
                "{} - {}",
                current_activity.duration().to_string(),
                if current_activity.is_work {
                    "Working"
                } else {
                    "Break"
                },
            )
        } else {
            format!("Left")
        };
        stdout.execute(SetTitle(&title))?;

        // Using `poll` for non-blocking read
        if crossterm::event::poll(std::time::Duration::from_millis(1000))? {
            let selection = match crossterm::event::read()? {
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('1'),
                    modifiers: KeyModifiers::NONE,
                }) => Some(1),
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('2'),
                    modifiers: KeyModifiers::NONE,
                }) => Some(2),
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('3'),
                    modifiers: KeyModifiers::NONE,
                }) => Some(3),
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('4'),
                    modifiers: KeyModifiers::NONE,
                }) => Some(4),
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('5'),
                    modifiers: KeyModifiers::NONE,
                }) => Some(5),
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('6'),
                    modifiers: KeyModifiers::NONE,
                }) => Some(6),
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('7'),
                    modifiers: KeyModifiers::NONE,
                }) => Some(7),
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('8'),
                    modifiers: KeyModifiers::NONE,
                }) => Some(8),
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('9'),
                    modifiers: KeyModifiers::NONE,
                }) => Some(9),

                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                })
                | crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    is_running = false;
                    None
                }
                _ => None,
            };

            // Run an activity
            if let Some(selection) = selection {
                assert!(selection != 0);
                if selection == 1 {
                    if day_entry.is_currently_working() {
                        day_entry.start_activitiy(ACTIVITY_NAME_LEAVE, false);
                    } else {
                        day_entry.start_activitiy(ACTIVITY_NAME_NON_PROJECT_WORK, true);
                    }
                } else {
                    let index = selection - 2;
                    if index < project_names_list.len() {
                        let project_name = &project_names_list[index];
                        let is_active = day_entry
                            .get_current_activity()
                            .map(|activity| activity.name == *project_name)
                            .unwrap_or(false);

                        if is_active {
                            day_entry.start_activitiy(ACTIVITY_NAME_NON_PROJECT_WORK, true);
                        } else {
                            day_entry.start_activitiy(project_name, true);
                        }
                    }
                }
            }
        }
    }

    stdout.execute(EnableLineWrap)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn reload_project_names() -> Vec<String> {
    const PROJECT_NAMES_FILEPATH: &str = "project_names.txt";
    if !path_exists(PROJECT_NAMES_FILEPATH) {
        let exampletext = format!(
            "Welcome to Thyme! :)
You can add your own project names here
by modifying '{}'!
Each project name will be its own line in '{}'.
Currently only up to 8 project names are supported.
Why not try out modifying '{}' now? 
(You don't need to close Thyme for this)
I will be waiting here",
            PROJECT_NAMES_FILEPATH, PROJECT_NAMES_FILEPATH, PROJECT_NAMES_FILEPATH
        );
        std::fs::write(&PROJECT_NAMES_FILEPATH, &exampletext).unwrap_or_else(|error| {
            panic!(
                "Could not write to '{}' - {}",
                &PROJECT_NAMES_FILEPATH, error
            )
        });
    }

    std::fs::read_to_string(PROJECT_NAMES_FILEPATH)
        .unwrap_or_else(|error| panic!("Could not read '{}' - {}", &PROJECT_NAMES_FILEPATH, error))
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.to_owned())
        .collect()
}

fn create_clear_screen(terminal_width: usize, terminal_heigth: usize) -> String {
    let mut result = String::new();
    for _line_index in 0..terminal_heigth {
        for _col_index in 0..terminal_width {
            write!(result, " ").unwrap();
        }
        writeln!(result, "").unwrap();
    }
    result
}

fn create_main_screen(
    day_entry: &DayEntry,
    project_list: &[String],
    terminal_width: usize,
    terminal_heigth: usize,
) -> String {
    let mut result = String::new();

    writeln!(
        result,
        "Hello! Today is {}",
        day_entry.datetime.format("%A %e. %b (%d.%m.%Y)"),
    )
    .unwrap();

    if let Some(checkin_time) = day_entry.first_checkin_time() {
        writeln!(result, "You started today at {}", checkin_time.to_string()).unwrap();
    } else {
        writeln!(result, "You haven't checked in today!").unwrap();
    }

    if day_entry.is_currently_working() {
        let last_work_activities = day_entry
            .activities
            .iter()
            .rev()
            .take_while(|activiy| activiy.is_work);

        let duration = last_work_activities
            .clone()
            .fold(TimeDuration::zero(), |acc, activity| {
                acc + activity.duration()
            });
        let start_time = last_work_activities.last().unwrap().time_start;

        writeln!(
            result,
            "You are working on since {} [{}]",
            start_time.to_string(),
            duration.to_string_composite(),
        )
        .unwrap();
    } else {
        if let Some(current_activity) = day_entry.get_current_activity() {
            writeln!(
                result,
                "You are on break since {} [{}]",
                current_activity.time_start.to_string(),
                current_activity.duration().to_string_composite(),
            )
            .unwrap();
        }
    }

    writeln!(result, "\n================================================").unwrap();

    writeln!(result, "{}", &write_durations_summary(day_entry)).unwrap();

    writeln!(result, "================================================").unwrap();

    if day_entry.is_currently_working() {
        writeln!(result, "(1) Take a break",).unwrap();
    } else {
        writeln!(result, "(1) Begin work",).unwrap();
    }
    for (index, project_name) in project_list.iter().enumerate().take(8) {
        let is_active = day_entry
            .get_current_activity()
            .map(|activity| activity.name == *project_name)
            .unwrap_or(false);

        writeln!(
            result,
            "({}) {} [{}]",
            index + 2,
            if is_active { "Stop" } else { "Begin" },
            project_name
        )
        .unwrap();
    }

    result
}

fn write_durations_summary(day_entry: &DayEntry) -> String {
    let mut result = String::new();

    let work_duration_total = day_entry.get_work_duration_total();
    let work_duration_projects = day_entry.get_work_duration_projects();
    let work_duration_non_project = day_entry.get_work_duration_non_project();
    let work_percent_projects = (100.0
        * (work_duration_projects.minutes as f32 / work_duration_total.minutes as f32))
        .round() as usize;
    let work_percent_non_project = 100 - work_percent_projects;
    writeln!(
        result,
        "Total work duration:        {} (100%)",
        work_duration_total.to_string_composite(),
    )
    .unwrap();
    writeln!(
        result,
        "  - Project activities:     {} ({: >3}%)",
        work_duration_projects.to_string_composite(),
        work_percent_projects
    )
    .unwrap();
    writeln!(
        result,
        "  - Non-Project activities: {} ({: >3}%)",
        work_duration_non_project.to_string_composite(),
        work_percent_non_project
    )
    .unwrap();
    writeln!(
        result,
        "Total break duration:       {}",
        day_entry.get_break_duration().to_string_composite(),
    )
    .unwrap();

    result
}

const ACTIVITY_NAME_NON_PROJECT_WORK: &str = "Work (Non-Project)";
const ACTIVITY_NAME_LEAVE: &str = "Leave";
const ACTIVITY_NAME_BREAK: &str = "Break";

struct DayEntry {
    pub datetime: DateTime<Local>,
    pub activities: Vec<Activity>,
}

impl DayEntry {
    pub fn create_empty() -> DayEntry {
        let datetime_today = Local::now();
        let result = DayEntry {
            activities: vec![],
            datetime: datetime_today,
        };
        result.write_back();
        result
    }

    pub fn load_or_create() -> DayEntry {
        let datetime_today = Local::now();
        let filepath_today = DayEntry::database_filepath_for_date(datetime_today);

        if path_exists(&filepath_today) {
            let content = std::fs::read_to_string(&filepath_today)
                .unwrap_or_else(|error| panic!("Could not read '{}' - {}", &filepath_today, error));

            let stamp_events: Vec<StampEvent> = content
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| StampEvent::from_string(line))
                .collect();

            let activities = DayEntry::create_activities_from_stamp_events(&stamp_events);
            DayEntry {
                datetime: datetime_today,
                activities,
            }
        } else {
            let result = DayEntry {
                activities: vec![Activity {
                    is_work: true,
                    name: ACTIVITY_NAME_NON_PROJECT_WORK.to_owned(),
                    time_start: datetime_today.to_timestamp(),
                    time_end: None,
                }],
                datetime: datetime_today,
            };
            result.write_back();
            result
        }
    }

    fn write_back(&self) {
        let filepath = DayEntry::database_filepath_for_date(self.datetime);
        let mut output = String::new();

        let stamp_events = DayEntry::create_stamp_events_from_activities(&self.activities);
        for stamp_event in &stamp_events {
            writeln!(output, "{}", stamp_event.to_string()).unwrap();
        }

        if !path_exists(&filepath) {
            std::fs::create_dir_all(path_without_filename(&filepath))
                .unwrap_or_else(|error| panic!("Could not crate path '{}' - {}", &filepath, error));
        }
        std::fs::write(&filepath, &output)
            .unwrap_or_else(|error| panic!("Could not write to '{}' - {}", &filepath, error));

        self.write_report();
    }

    fn write_report(&self) {
        let report = self.generate_report();

        let report_filepath = DayEntry::report_filepath_for_date(self.datetime);
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
        let checkin_date = self.datetime;

        writeln!(
            result,
            "Report for {}\n",
            checkin_date.format("%A %e. %b (%d.%m.%Y)"),
        )
        .unwrap();

        writeln!(result, "\nActivity Durations:").unwrap();
        writeln!(result, "=====================\n").unwrap();

        let activity_names: HashSet<String> = self
            .activities
            .iter()
            .filter(|activity| activity.is_work)
            .map(|activity| activity.name.clone())
            .collect();

        let mut activity_names_and_durations: Vec<_> = activity_names
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

        activity_names_and_durations.sort_by_key(|(_activity_name, duration)| {
            // NOTE: This forces the sort to be descending
            -duration.minutes
        });

        for (activity_name, duration) in activity_names_and_durations.into_iter() {
            writeln!(
                result,
                "{} - {}",
                duration.to_string_composite(),
                activity_name
            )
            .unwrap();
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

    fn start_activitiy(&mut self, name: &str, is_work: bool) {
        let timestamp_now = Local::now().to_timestamp();

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

    fn is_currently_working(&self) -> bool {
        if let Some(activity) = self.get_current_activity() {
            activity.is_work
        } else {
            false
        }
    }

    fn get_current_activity(&self) -> Option<&Activity> {
        self.activities.last()
    }

    fn get_current_activity_mut(&mut self) -> Option<&mut Activity> {
        self.activities.last_mut()
    }

    fn first_checkin_time(&self) -> Option<TimeStamp> {
        self.activities.first().map(|activity| activity.time_start)
    }

    fn get_work_duration_total(&self) -> TimeDuration {
        self.activities
            .iter()
            .filter(|activity| activity.is_work)
            .fold(TimeDuration::zero(), |acc, activity| {
                acc + activity.duration()
            })
    }

    fn get_work_duration_projects(&self) -> TimeDuration {
        self.activities
            .iter()
            .filter(|activity| activity.is_work)
            .filter(|activity| activity.name != ACTIVITY_NAME_NON_PROJECT_WORK)
            .fold(TimeDuration::zero(), |acc, activity| {
                acc + activity.duration()
            })
    }

    fn get_work_duration_non_project(&self) -> TimeDuration {
        self.activities
            .iter()
            .filter(|activity| activity.is_work)
            .filter(|activity| activity.name == ACTIVITY_NAME_NON_PROJECT_WORK)
            .fold(TimeDuration::zero(), |acc, activity| {
                acc + activity.duration()
            })
    }

    fn get_break_duration(&self) -> TimeDuration {
        self.activities
            .iter()
            .filter(|activity| !activity.is_work)
            .filter(|activity| activity.time_end.is_some())
            .fold(TimeDuration::zero(), |acc, activity| {
                acc + activity.duration()
            })
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

    fn database_filepath_for_date(datetime: DateTime<Local>) -> String {
        format!("database/{}.txt", datetime.format("%Y_%m_%d__%b_%A"))
    }
    fn report_filepath_for_date(datetime: DateTime<Local>) -> String {
        format!(
            "database/{}__report.txt",
            datetime.format("%Y_%m_%d__%b_%A")
        )
    }
    fn report_filepath_default() -> String {
        "today__report.txt".to_owned()
    }
}

trait DateTimeHelper {
    fn to_date(&self) -> Date;
    fn to_timestamp(&self) -> TimeStamp;
}

impl DateTimeHelper for DateTime<Local> {
    fn to_date(&self) -> Date {
        Date::new(self.year() as u32, self.month(), self.day())
    }

    fn to_timestamp(&self) -> TimeStamp {
        TimeStamp::new(self.hour(), self.minute())
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
            self.duration().to_string_composite(),
            self.name,
        )
    }

    pub fn duration(&self) -> TimeDuration {
        if let Some(end) = self.time_end {
            end - self.time_start
        } else {
            Local::now().to_timestamp() - self.time_start
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
            StampEvent::Begin(timestamp, _) => *timestamp,
            StampEvent::Leave(timestamp) => *timestamp,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    fn new(year: u32, month: u32, day: u32) -> Date {
        assert!(0 < month && month <= 12);
        assert!(0 < day && day <= 31);
        Date { year, month, day }
    }

    fn to_string_ymd(&self, separator: &str) -> String {
        format!(
            "{}{}{:02}{}{:02}",
            self.year, separator, self.month, separator, self.day
        )
    }

    fn to_string_dmy(&self, separator: &str) -> String {
        format!(
            "{}{}{:02}{}{:02}",
            self.day, separator, self.month, separator, self.year
        )
    }

    fn from_string_ymd(input: &str) -> Date {
        let separators = ["_", "-", "."];
        for separator in &separators {
            if input.contains(separator) {
                let mut parts = input.split_terminator(separator);
                let year = parts
                    .next()
                    .unwrap_or_else(|| panic!("The string '{}' is not a valid date", input))
                    .parse()
                    .unwrap_or_else(|error| {
                        panic!("The string '{}' is not a valid date: {}", input, error)
                    });

                let month = parts
                    .next()
                    .unwrap_or_else(|| panic!("The string '{}' is not a valid date", input))
                    .parse()
                    .unwrap_or_else(|error| {
                        panic!("The string '{}' is not a valid date: {}", input, error)
                    });

                let day = parts
                    .next()
                    .unwrap_or_else(|| panic!("The string '{}' is not a valid date", input))
                    .parse()
                    .unwrap_or_else(|error| {
                        panic!("The string '{}' is not a valid date: {}", input, error)
                    });

                assert!(
                    parts.next().is_none(),
                    "The string '{}' is not a valid date",
                    input
                );

                return Date::new(year, month, day);
            }
        }
        panic!("The string '{}' is not a valid date", input)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeStamp {
    pub hours: u32,
    pub minutes: u32,
}

impl TimeStamp {
    pub fn new(hours: u32, minutes: u32) -> TimeStamp {
        assert!(hours < 24);
        assert!(minutes < 60);
        TimeStamp { hours, minutes }
    }

    pub fn from_string(input: &str) -> TimeStamp {
        let mut parts = input.split(":");
        let hours = parts
            .next()
            .unwrap_or_else(|| panic!("The string '{}' is not a valid timestamp", input))
            .parse()
            .unwrap_or_else(|error| {
                panic!("The string '{}' is not a valid timestamp: {}", input, error)
            });
        let minutes = parts
            .next()
            .unwrap_or_else(|| panic!("The string '{}' is not a valid timestamp", input))
            .parse()
            .unwrap_or_else(|error| {
                panic!("The string '{}' is not a valid timestamp: {}", input, error)
            });

        assert!(
            parts.next().is_none(),
            "The string '{}' is not a valid timestamp",
            input
        );
        TimeStamp::new(hours, minutes)
    }

    pub fn to_string(&self) -> String {
        format!("{:02}:{:02}", self.hours, self.minutes)
    }
}

use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;

impl Add<TimeStamp> for TimeStamp {
    type Output = TimeDuration;
    #[inline]
    fn add(self, rhs: TimeStamp) -> TimeDuration {
        TimeDuration {
            minutes: (self.hours * 60 + self.minutes + rhs.hours * 60 + rhs.minutes) as i32,
        }
    }
}
impl Sub<TimeStamp> for TimeStamp {
    type Output = TimeDuration;
    #[inline]
    fn sub(self, rhs: TimeStamp) -> TimeDuration {
        TimeDuration {
            minutes: (self.hours * 60 + self.minutes) as i32
                - (rhs.hours * 60 + rhs.minutes) as i32,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TimeDuration {
    pub minutes: i32,
}

impl TimeDuration {
    pub fn zero() -> TimeDuration {
        TimeDuration { minutes: 0 }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}{}",
            if self.minutes < 0 { "-" } else { "" },
            TimeStamp::new(
                self.minutes.abs() as u32 / 60,
                self.minutes.abs() as u32 % 60
            )
            .to_string()
        )
    }

    pub fn to_string_hour_fraction(&self) -> String {
        format!(
            "{}{:2.2}",
            if self.minutes < 0 { "-" } else { "" },
            self.minutes.abs() as f32 / 60.0
        )
    }

    pub fn to_string_composite(&self) -> String {
        format!("{} ({}h)", self.to_string(), self.to_string_hour_fraction())
    }
}

impl Add<TimeDuration> for TimeDuration {
    type Output = TimeDuration;
    #[inline]
    fn add(self, rhs: TimeDuration) -> TimeDuration {
        TimeDuration {
            minutes: self.minutes + rhs.minutes,
        }
    }
}
impl AddAssign<TimeDuration> for TimeDuration {
    #[inline]
    fn add_assign(&mut self, rhs: TimeDuration) {
        *self = *self + rhs
    }
}
impl Sub<TimeDuration> for TimeDuration {
    type Output = TimeDuration;
    #[inline]
    fn sub(self, rhs: TimeDuration) -> TimeDuration {
        TimeDuration {
            minutes: self.minutes - rhs.minutes,
        }
    }
}
impl SubAssign<TimeDuration> for TimeDuration {
    #[inline]
    fn sub_assign(&mut self, rhs: TimeDuration) {
        *self = *self - rhs
    }
}
