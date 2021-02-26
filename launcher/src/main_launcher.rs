#[macro_use]
extern crate crossterm;

use chrono::{prelude::*, Local};
use crossterm::cursor::{self, EnableBlinking};
use crossterm::event::{read, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use ct_lib_core::{path_exists, path_without_filename};

use std::fmt::Write;
use std::io::stdout;

fn main() {
    let day_entry = DayEntry::new();
    let checkin_date = day_entry.datetime;
    let checkin_time = day_entry.stamp_events[0].timestamp();

    let work_duration = day_entry.get_work_duration();
    let (break_duration_confirmed, duration_since_last_checkout) =
        day_entry.get_break_duration_and_duration_since_last_checkout();
    println!(
        "Hello! Today is {} - you started at {}",
        checkin_date.format("%A %e. %b (%d.%m.%Y)"),
        checkin_time.to_string()
    );
    println!(
        "Work duration:            {} ({}h)",
        work_duration.to_string(),
        work_duration.to_string_hour_fraction()
    );

    println!(
        "Break duration:           {} ({}h)",
        break_duration_confirmed.to_string(),
        break_duration_confirmed.to_string_hour_fraction(),
    );

    if duration_since_last_checkout.minutes != 0 {
        println!(
            "Time since last checkout: {} ({}h)",
            duration_since_last_checkout.to_string(),
            duration_since_last_checkout.to_string_hour_fraction(),
        );
    }

    return;

    let mut stdout = stdout();
    //going into raw mode
    enable_raw_mode().unwrap();

    //clearing the screen, going to top left corner and printing welcoming message
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0), Print(r#"ctrl + q to exit, ctrl + h to print "Hello world", alt + t to print "crossterm is cool""#))
            .unwrap();

    //key detection
    loop {
        //going to top left corner
        execute!(stdout, cursor::MoveTo(0, 0)).unwrap();

        //matching the key
        match read().unwrap() {
            crossterm::event::Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::CONTROL,
                //clearing the screen and printing our message
            }) => execute!(stdout, Clear(ClearType::All), Print("Hello world!")).unwrap(),
            crossterm::event::Event::Key(KeyEvent {
                code: KeyCode::Char('t'),
                modifiers: KeyModifiers::ALT,
            }) => execute!(stdout, Clear(ClearType::All), Print("crossterm is cool")).unwrap(),
            crossterm::event::Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            }) => break,
            _ => (),
        }
    }

    //disabling raw mode
    disable_raw_mode().unwrap();
}

struct DayEntry {
    pub stamp_events: Vec<StampEvent>,
    pub datetime: DateTime<Local>,
}

impl DayEntry {
    pub fn new() -> DayEntry {
        let datetime_today = Local::now();
        let filepath_today = DayEntry::filepath_for_date(datetime_today);

        if path_exists(&filepath_today) {
            let content = std::fs::read_to_string(&filepath_today)
                .unwrap_or_else(|error| panic!("Could not read '{}' - {}", &filepath_today, error));

            let stamp_events = content
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| StampEvent::from_string(line))
                .collect();

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
        let filepath = DayEntry::filepath_for_date(self.datetime);
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
    }

    fn get_work_duration(&self) -> TimeDuration {
        let mut last_checkin_stamp = None;
        let mut duration_checked_in = TimeDuration::zero();
        for event in self.stamp_events.iter() {
            match event {
                StampEvent::CheckIn(timestamp) => {
                    assert!(
                        last_checkin_stamp.is_none(),
                        "Got a duplicate checkin at {}",
                        timestamp.to_string()
                    );
                    last_checkin_stamp = Some(timestamp);
                }
                StampEvent::CheckOut(timestamp) => {
                    assert!(
                        last_checkin_stamp.is_some(),
                        "Got a checkout at {} without checkin",
                        timestamp.to_string()
                    );
                    duration_checked_in += *timestamp - *last_checkin_stamp.unwrap();
                    last_checkin_stamp = None;
                }
            }
        }
        if let Some(&last_checkin_stamp) = last_checkin_stamp {
            duration_checked_in += Local::now().to_timestamp() - last_checkin_stamp;
        }
        duration_checked_in
    }

    fn get_break_duration_and_duration_since_last_checkout(&self) -> (TimeDuration, TimeDuration) {
        let mut last_checkout_stamp: Option<TimeStamp> = None;
        let mut duration_checked_out = TimeDuration::zero();
        for event in self.stamp_events.iter() {
            match event {
                StampEvent::CheckIn(timestamp) => {
                    if last_checkout_stamp.is_none() {
                        continue;
                    }
                    duration_checked_out += *timestamp - last_checkout_stamp.unwrap();
                    last_checkout_stamp = None;
                }
                StampEvent::CheckOut(timestamp) => {
                    assert!(
                        last_checkout_stamp.is_none(),
                        "Got a duplicate checkout at {}",
                        timestamp.to_string()
                    );
                    last_checkout_stamp = Some(*timestamp);
                }
            }
        }
        let mut duration_since_last_checkout = TimeDuration::zero();
        if let Some(last_checkout_stamp) = last_checkout_stamp {
            duration_since_last_checkout += Local::now().to_timestamp() - last_checkout_stamp;
        }
        (duration_checked_out, duration_since_last_checkout)
    }

    fn filepath_for_date(datetime: DateTime<Local>) -> String {
        format!("database/{}.txt", datetime.format("%Y_%m_%d__%b_%A"))
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

#[derive(Debug, Copy, Clone)]
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
