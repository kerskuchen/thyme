#[macro_use]
extern crate crossterm;

use chrono::{prelude::*, Local};
use crossterm::event::{read, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{
    cursor::{self, EnableBlinking},
    ExecutableCommand,
};
use ct_lib_core::{path_exists, path_without_filename, serde::de::value::StringDeserializer};

use std::fmt::Write;
use std::io::stdout;

fn main() -> crossterm::Result<()> {
    let mut day_entry = DayEntry::load();

    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;

    let mut is_running = true;
    while is_running {
        match crossterm::event::read()? {
            crossterm::event::Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::NONE,
            }) => {
                if day_entry.is_working() {
                    day_entry.check_out();
                } else {
                    day_entry.check_in();
                }
            }
            crossterm::event::Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
            }) => is_running = false,
            _ => (),
        }

        let main_screen: String = create_main_screen(&day_entry);
        stdout
            .execute(Clear(ClearType::All))?
            .execute(cursor::MoveTo(0, 0))?
            .execute(Print(&main_screen))?;

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn create_main_screen(day_entry: &DayEntry) -> String {
    let mut result = String::new();
    let checkin_date = day_entry.datetime;
    let checkin_time = day_entry.first_checkin_time();

    writeln!(
        result,
        "Hello! Today is {}\n",
        checkin_date.format("%A %e. %b (%d.%m.%Y)"),
    )
    .unwrap();

    writeln!(result, "Time pairs:").unwrap();
    for pair in day_entry.get_time_pairs() {
        writeln!(result, "{}", pair.to_string()).unwrap();
    }
    writeln!(result, "").unwrap();

    writeln!(result, "You started at {}", checkin_time.to_string()).unwrap();
    match day_entry.get_last_check_event() {
        StampEvent::CheckIn(timestamp) => {
            writeln!(result, "You are working since {}", timestamp.to_string()).unwrap();
        }
        StampEvent::CheckOut(timestamp) => {
            writeln!(result, "You are on break since {}", timestamp.to_string()).unwrap();
        }
    }
    writeln!(result, "=======================================").unwrap();
    let work_duration = day_entry.get_work_duration();
    let break_duration = day_entry.get_break_duration();
    let duration_since_last_checkout = day_entry.get_duration_since_last_leave();

    writeln!(
        result,
        "Total work duration:      {}",
        work_duration.to_string_composite(),
    )
    .unwrap();

    writeln!(
        result,
        "Total break duration:     {}",
        break_duration.to_string_composite(),
    )
    .unwrap();

    if let Some(duration_since_last_checkout) = duration_since_last_checkout {
        writeln!(
            result,
            "Time since last checkout: {}",
            duration_since_last_checkout.to_string_composite(),
        )
        .unwrap();
    }

    result
}

struct DayEntry {
    pub stamp_events: Vec<StampEvent>,
    pub datetime: DateTime<Local>,
}

impl DayEntry {
    pub fn load() -> DayEntry {
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

            assert!(
                !stamp_events.is_empty(),
                "No stamp events found in '{}'",
                filepath_today,
            );

            DayEntry {
                stamp_events,
                datetime: datetime_today,
            }
        } else {
            let result = DayEntry {
                stamp_events: vec![StampEvent::CheckIn(datetime_today.to_timestamp())],
                datetime: datetime_today,
            };
            result.write_back();
            result
        }
    }

    fn write_back(&self) {
        let filepath = DayEntry::database_filepath_for_date(self.datetime);
        let mut output = String::new();
        for stamp_event in &self.stamp_events {
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
        let report_filepath_default = DayEntry::report_filepath_default(self.datetime);
        std::fs::write(&report_filepath_default, &report).unwrap_or_else(|error| {
            panic!("Could not write to '{}' - {}", &report_filepath, error)
        });
    }

    fn generate_report(&self) -> String {
        let mut result = String::new();
        let checkin_date = self.datetime;
        let checkin_time = self.first_checkin_time();

        writeln!(
            result,
            "Report for {}\n",
            checkin_date.format("%A %e. %b (%d.%m.%Y)"),
        )
        .unwrap();

        writeln!(result, "Time pairs:").unwrap();
        for pair in self.get_time_pairs() {
            writeln!(result, "{}", pair.to_string()).unwrap();
        }
        writeln!(result, "").unwrap();

        writeln!(result, "You started at {}", checkin_time.to_string()).unwrap();
        match self.get_last_check_event() {
            StampEvent::CheckIn(timestamp) => {
                writeln!(result, "You are working since {}", timestamp.to_string()).unwrap();
            }
            StampEvent::CheckOut(timestamp) => {
                writeln!(result, "You are on break since {}", timestamp.to_string()).unwrap();
            }
        }
        writeln!(result, "=======================================").unwrap();
        let work_duration = self.get_work_duration();
        let break_duration = self.get_break_duration();
        let duration_since_last_checkout = self.get_duration_since_last_leave();

        writeln!(
            result,
            "Total work duration:      {}",
            work_duration.to_string_composite(),
        )
        .unwrap();

        writeln!(
            result,
            "Total break duration:     {}",
            break_duration.to_string_composite(),
        )
        .unwrap();

        if let Some(duration_since_last_checkout) = duration_since_last_checkout {
            writeln!(
                result,
                "Time since last checkout: {}",
                duration_since_last_checkout.to_string_composite(),
            )
            .unwrap();
        }

        result
    }

    fn check_in(&mut self) {
        assert!(
            !self.is_working(),
            "Trying to check in while already being checked in"
        );
        let datetime_today = Local::now();
        self.stamp_events
            .push(StampEvent::CheckIn(datetime_today.to_timestamp()));
        self.remove_zero_sized_pairs();
        self.write_back();
    }

    fn check_out(&mut self) {
        assert!(
            self.is_working(),
            "Trying to check out while already being checked out"
        );
        let datetime_today = Local::now();
        self.stamp_events
            .push(StampEvent::CheckOut(datetime_today.to_timestamp()));
        self.remove_zero_sized_pairs();
        self.write_back();
    }

    fn remove_zero_sized_pairs(&mut self) {
        todo!();
    }

    fn is_working(&self) -> bool {
        match self.stamp_events.last().unwrap() {
            StampEvent::CheckIn(_) => true,
            StampEvent::CheckOut(_) => false,
        }
    }

    fn get_last_check_event(&self) -> StampEvent {
        for event in self.stamp_events.iter().rev() {
            match event {
                StampEvent::CheckIn(_) | StampEvent::CheckOut(_) => {}
                _ => continue,
            }
            return event.clone();
        }
        unreachable!()
    }

    fn first_checkin_time(&self) -> TimeStamp {
        self.stamp_events[0].timestamp()
    }

    fn get_time_pairs(&self) -> Vec<TimePair> {
        let mut stamp_pairs = Vec::new();
        let mut current_pair: Option<TimePair> = None;
        for event in self.stamp_events.iter() {
            match event {
                StampEvent::CheckIn(timestamp) => {
                    if current_pair.is_some() {
                        match current_pair.as_ref().unwrap().stamp_type {
                            PairType::Break => {}
                            PairType::Work => {
                                panic!("Got a duplicate checkin at {}", timestamp.to_string())
                            }
                        }
                        current_pair.as_mut().unwrap().end = Some(*timestamp);
                        stamp_pairs.push(current_pair.take().unwrap());
                    }

                    // Start new work pair
                    current_pair = Some(TimePair {
                        stamp_type: PairType::Work,
                        start: *timestamp,
                        end: None,
                    });
                }
                StampEvent::CheckOut(timestamp) => {
                    if current_pair.is_some() {
                        match current_pair.as_ref().unwrap().stamp_type {
                            PairType::Work => {}
                            PairType::Break => {
                                panic!("Got a duplicate checkout at {}", timestamp.to_string())
                            }
                        }
                        current_pair.as_mut().unwrap().end = Some(*timestamp);
                        stamp_pairs.push(current_pair.take().unwrap());
                    }

                    // Start new idle pair
                    current_pair = Some(TimePair {
                        stamp_type: PairType::Break,
                        start: *timestamp,
                        end: None,
                    });
                }
            }
        }
        if let Some(current_pair) = current_pair {
            stamp_pairs.push(current_pair);
        }
        stamp_pairs
    }

    fn get_work_duration(&self) -> TimeDuration {
        let stamp_pairs = self.get_time_pairs();
        stamp_pairs
            .iter()
            .filter(|pair| pair.is_work())
            .fold(TimeDuration::zero(), |acc, pair| acc + pair.duration())
    }

    fn get_break_duration(&self) -> TimeDuration {
        let stamp_pairs = self.get_time_pairs();
        stamp_pairs
            .iter()
            .filter(|pair| !pair.is_work())
            .filter(|pair| pair.end.is_some())
            .fold(TimeDuration::zero(), |acc, pair| acc + pair.duration())
    }

    fn get_duration_since_last_leave(&self) -> Option<TimeDuration> {
        let stamp_pairs = self.get_time_pairs();
        let last_pair = stamp_pairs.last().unwrap();
        if !last_pair.is_work() && last_pair.end.is_none() {
            Some(last_pair.duration())
        } else {
            None
        }
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
    fn report_filepath_default(datetime: DateTime<Local>) -> String {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PairType {
    Break,
    Work,
}

#[derive(Debug, Copy, Clone)]
pub struct TimePair {
    pub stamp_type: PairType,
    pub start: TimeStamp,
    pub end: Option<TimeStamp>,
}

impl TimePair {
    pub fn is_work(&self) -> bool {
        match self.stamp_type {
            PairType::Break => false,
            PairType::Work => true,
        }
    }
    pub fn to_string(&self) -> String {
        match self.stamp_type {
            PairType::Break => {
                format!(
                    "{} [{}] - {}",
                    self.to_string_time_range(),
                    self.duration().to_string_composite(),
                    if self.end.is_some() { "break" } else { "leave" }
                )
            }
            PairType::Work => {
                format!(
                    "{} [{}] - work",
                    self.to_string_time_range(),
                    self.duration().to_string_composite()
                )
            }
        }
    }

    pub fn to_string_time_range(&self) -> String {
        if let Some(end) = self.end {
            format!("{} - {}", self.start.to_string(), end.to_string())
        } else {
            format!("{} - {}", self.start.to_string(), "[now]")
        }
    }

    pub fn duration(&self) -> TimeDuration {
        if let Some(end) = self.end {
            end - self.start
        } else {
            Local::now().to_timestamp() - self.start
        }
    }
}

#[derive(Debug, Clone)]
enum StampEvent {
    CheckIn(TimeStamp),
    CheckOut(TimeStamp),
    // TaskStart(Local, String),
    // TaskEnd(Local, String),
}

impl StampEvent {
    fn timestamp(&self) -> TimeStamp {
        match self {
            StampEvent::CheckIn(timestamp) => *timestamp,
            StampEvent::CheckOut(timestamp) => *timestamp,
        }
    }

    fn to_string(&self) -> String {
        match self {
            StampEvent::CheckIn(timestamp) => format!("{} - CheckIn", timestamp.to_string()),
            StampEvent::CheckOut(timestamp) => format!("{} - CheckOut", timestamp.to_string()),
        }
    }

    fn from_string(input: &str) -> StampEvent {
        let delimiter_pos = input
            .find('-')
            .unwrap_or_else(|| panic!("The string '{}' is not a valid stamp event", input));

        let (left, right) = input.split_at(delimiter_pos);
        let timestamp = TimeStamp::from_string(left.trim());
        let right_replaced = right.to_owned().replace("-", "");
        let right_trimmed = right_replaced.trim();
        let right = right_trimmed;

        if right.starts_with("CheckIn") {
            return StampEvent::CheckIn(timestamp);
        }
        if right.starts_with("CheckOut") {
            return StampEvent::CheckOut(timestamp);
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
