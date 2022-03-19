mod dayentry;
mod time;

use dayentry::{
    write_durations_summary, DayEntry, ACTIVITY_NAME_LEAVE, ACTIVITY_NAME_NON_SPECIFIC_WORK,
};
use time::{DateTimeHelper, TimeDuration, TimeStamp};

use ct_lib_core::path_exists;

use chrono::{prelude::*, Local};
use crossterm::{
    cursor::{self},
    event::{KeyCode, KeyEvent, KeyModifiers},
    style::Print,
    terminal::{DisableLineWrap, EnableLineWrap, SetTitle},
    ExecutableCommand, QueueableCommand,
};

use std::fmt::Write;

fn main() -> crossterm::Result<()> {
    ct_lib_core::panic_set_hook_wait_for_keypress();

    let mut day_entry = DayEntry::load_or_create();

    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    stdout.execute(DisableLineWrap)?;

    let mut preferred_working_time = TimeDuration { minutes: 8 * 60 };
    let mut previous_time = time::get_current_datetime();
    let mut is_running = true;
    while is_running {
        let activity_names_list = reload_activity_names();

        day_entry.hotreload_external_changes();

        // Write changes every minute
        let current_time = time::get_current_datetime();
        if (current_time - previous_time).num_minutes() > 0 {
            // One minute has passed
            day_entry.write_back();
            if current_time.date() != previous_time.date() {
                // A whole day has passed - we need to create a new entry
                day_entry = DayEntry::create_empty();
            }
            previous_time = current_time;
        }

        let (terminal_width, terminal_height) = crossterm::terminal::size().unwrap_or((100, 30));
        let terminal_width = (terminal_width - 2) as usize;
        let terminal_height = (terminal_height.min(30) - 2) as usize;

        let clear_screen = create_clear_screen(terminal_width, terminal_height);
        let sprite_screen = create_sprite_screen(&day_entry, terminal_width, terminal_height);
        let main_screen = create_main_screen(
            &day_entry,
            &activity_names_list,
            preferred_working_time,
            terminal_width,
            terminal_height,
        );

        let title = {
            let blink = Local::now().second() % 2 == 0;
            if day_entry.get_current_activity().is_some() {
                if day_entry.is_currently_working() {
                    format!(
                        "{} {}",
                        day_entry
                            .get_work_duration_total()
                            .to_string_blinking_shortened(blink),
                        "Work total",
                    )
                } else {
                    format!(
                        "{} {}",
                        day_entry
                            .get_non_work_duration()
                            .to_string_blinking_shortened(blink),
                        "Break total"
                    )
                }
            } else {
                format!("Not Checked in today")
            }
        };

        use std::io::Write;
        stdout
            .queue(SetTitle(&title))?
            .queue(cursor::MoveTo(0, 0))?
            //.queue(Clear(ClearType::All))? // This just scrolls down on Windows and looks glitchy
            .queue(Print(&clear_screen))?
            .queue(cursor::MoveTo(0, 0))?
            .queue(Print(&sprite_screen))?
            .queue(cursor::MoveTo(0, 0))?
            .queue(Print(&main_screen))?;
        stdout.flush()?;

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
                    code: KeyCode::Char('x'),
                    modifiers: KeyModifiers::NONE,
                }) => Some(0),

                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('+'),
                    modifiers: KeyModifiers::NONE,
                }) => {
                    preferred_working_time.minutes =
                        i32::min(preferred_working_time.minutes + 1, 10 * 60);
                    None
                }
                crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('-'),
                    modifiers: KeyModifiers::NONE,
                }) => {
                    preferred_working_time.minutes =
                        i32::max(preferred_working_time.minutes - 15, 4 * 60);
                    None
                }

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
                if selection == 0 {
                    if day_entry.is_currently_working() {
                        day_entry.start_activitiy(ACTIVITY_NAME_LEAVE, false);
                    } else {
                        day_entry.start_activitiy(ACTIVITY_NAME_NON_SPECIFIC_WORK, true);
                    }
                } else {
                    let index = selection - 1;
                    if index < activity_names_list.len() {
                        let activity_name = &activity_names_list[index];
                        let is_active = day_entry
                            .get_current_activity()
                            .map(|activity| activity.name == *activity_name)
                            .unwrap_or(false);

                        if is_active {
                            day_entry.start_activitiy(ACTIVITY_NAME_NON_SPECIFIC_WORK, true);
                        } else {
                            day_entry.start_activitiy(activity_name, true);
                        }
                    }
                }
                // Something changed
                day_entry.write_back();
            }
        }
    }

    stdout.execute(EnableLineWrap)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn reload_activity_names() -> Vec<String> {
    const ACTIVITY_NAMES_FILEPATH: &str = "activity_names.txt";

    // Auto-rename old project names file name
    if path_exists("project_names.txt") {
        std::fs::rename("project_names.txt", ACTIVITY_NAMES_FILEPATH)
            .expect("Could not rename old 'project_names.txt' to 'activity_names.txt'");
    }

    if !path_exists(ACTIVITY_NAMES_FILEPATH) {
        let exampletext = format!(
            "Welcome to Thyme! :)
You can add your own activity names here
by modifying '{}'!
Each activity name will be its own line in '{}'.
Currently only up to 9 activity names are supported.
Why not try out modifying '{}' now? 
(You don't need to close Thyme for this)
I will be waiting here",
            ACTIVITY_NAMES_FILEPATH, ACTIVITY_NAMES_FILEPATH, ACTIVITY_NAMES_FILEPATH
        );
        std::fs::write(&ACTIVITY_NAMES_FILEPATH, &exampletext).unwrap_or_else(|error| {
            panic!(
                "Could not write to '{}' - {}",
                &ACTIVITY_NAMES_FILEPATH, error
            )
        });
    }

    let actitivities: Vec<String> = std::fs::read_to_string(ACTIVITY_NAMES_FILEPATH)
        .unwrap_or_else(|error| panic!("Could not read '{}' - {}", &ACTIVITY_NAMES_FILEPATH, error))
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            assert!(
                line.len() <= 70,
                "Activity name [{}] is too long - please make it shorter than 70 character",
                line
            );
            line
        })
        .map(|line| line.to_owned())
        .collect();

    assert!(
        actitivities.len() < 10,
        "Activitiy list is too long, only up to 9 activities are supported"
    );
    actitivities
}

fn create_sprite_screen(
    day_entry: &DayEntry,
    _terminal_width: usize,
    _terminal_height: usize,
) -> String {
    const SPRITE_WORKING: [&str; 2] = [
        "................
.....WWWWWW.....
....W......W....
...W..W.W...W...
..W..........W..
..W..........W..
...W........W...
....WWWWWWWW....",
        "................
................
................
.....WWWWWW.....
...WW......WW...
..W...W.W....W..
.W............W.
..WWWWWWWWWWWW..",
    ];

    const SPRITE_BREAK: [&str; 2] = [
        ".............ZZ.
................
............ZZ..
.....ZZZZZZ.....
...ZZ......ZZ...
..Z.ZZZ.ZZZ..Z..
.Z............Z.
..ZZZZZZZZZZZZ..",
        "................
............ZZ..
................
....ZZZZZZZZ....
..ZZ........ZZ..
.Z..ZZZ.ZZZ...Z.
Z..............Z
.ZZZZZZZZZZZZZZ.",
    ];

    let sprite_index = (Local::now().second() % 2) as usize;
    let sprite = if day_entry.is_currently_working() {
        SPRITE_WORKING[sprite_index].to_owned()
    } else {
        SPRITE_BREAK[sprite_index].to_owned()
    };

    let sprite = sprite.replace(".", " ").replace("W", "@").replace("Z", "@");
    let padding_left = 55;

    let mut result = String::new();

    writeln!(result, "\n\n\n").unwrap();
    for _ in 0..padding_left {
        write!(result, " ").unwrap();
    }
    if day_entry.get_leave_duration().is_some() {
        writeln!(result, "(Don't forget to log your hours!)",).unwrap();
    } else {
        writeln!(result, "").unwrap();
    }

    for line in sprite.lines() {
        for _ in 0..padding_left {
            write!(result, " ").unwrap();
        }
        writeln!(result, "{}", line).unwrap();
    }
    result
}

fn create_clear_screen(terminal_width: usize, terminal_height: usize) -> String {
    let mut result = String::new();
    for _line_index in 0..terminal_height {
        for _col_index in 0..terminal_width {
            write!(result, " ").unwrap();
        }
        writeln!(result, "").unwrap();
    }
    result
}

fn create_main_screen(
    day_entry: &DayEntry,
    activity_names_list: &[String],
    preferred_working_time: TimeDuration,
    _terminal_width: usize,
    _terminal_heigth: usize,
) -> String {
    let mut result = String::new();

    write!(
        result,
        "Today is {} -- ",
        day_entry.date.format("%A %e. %b (%d.%m.%Y)"),
    )
    .unwrap();

    if let Some(checkin_time) = day_entry.first_checkin_time() {
        writeln!(result, "You started at {}", checkin_time.to_string()).unwrap();
        let time_left = day_entry.get_time_left_for_the_day(
            preferred_working_time,
            mandatory_break_time_for_working_time(preferred_working_time),
        );

        if time_left.minutes >= 0 {
            let finished_time = {
                let current_time = time::get_current_datetime().to_timestamp();
                let mut hours = current_time.hours;
                let mut minutes = current_time.minutes + time_left.minutes as u32;
                while minutes >= 60 {
                    minutes -= 60;
                    hours += 1;
                }
                hours = hours % 24;

                TimeStamp::new(hours, minutes)
            };
            writeln!(
                result,
                "For a preferred work time of {} you will be finished at {} ({} left)",
                preferred_working_time.to_string(),
                finished_time.to_string(),
                time_left.to_string()
            )
            .unwrap();
        } else {
            writeln!(
                result,
                "You already finished your preferred work time of {} with overtime of {}",
                preferred_working_time.to_string(),
                (TimeDuration::zero() - time_left).to_string()
            )
            .unwrap();
        }
    } else {
        writeln!(result, "You haven't checked in today!").unwrap();
    }

    if let Some(current_activity) = day_entry.get_current_activity() {
        writeln!(
            result,
            "You are {} since {} [{}]",
            if current_activity.is_work {
                format!("doing [{}]", &current_activity.name)
            } else {
                "checked out".to_owned()
            },
            current_activity.time_start.to_string(),
            current_activity.duration().to_string(),
        )
        .unwrap();
    } else {
        writeln!(result, "").unwrap();
    }

    writeln!(
        result,
        "\n=================================================\n"
    )
    .unwrap();

    writeln!(result, "{}", &write_durations_summary(day_entry)).unwrap();

    writeln!(
        result,
        "=================================================\n"
    )
    .unwrap();

    if day_entry.is_currently_working() {
        writeln!(result, "(x) Take a break\n",).unwrap();
    } else {
        writeln!(result, "<x> Begin work\n",).unwrap();
    }

    let activity_durations = day_entry.get_activity_durations();
    let mut lines = Vec::new();
    for (index, activity_name) in activity_names_list.iter().enumerate().take(9) {
        let is_active = day_entry
            .get_current_activity()
            .map(|activity| activity.name == *activity_name)
            .unwrap_or(false);
        let duration = activity_durations
            .get(activity_name)
            .unwrap_or(&TimeDuration::zero())
            .to_string();
        lines.push((
            duration,
            if is_active {
                format!("<{}> {} [{}]", index + 1, "Stop ", activity_name,)
            } else {
                format!("({}) {} [{}]", index + 1, "Begin", activity_name,)
            },
        ));
    }

    let max_line_len = lines
        .iter()
        .map(|(_activitiy_name, line)| line.len())
        .max()
        .unwrap_or(0);

    for (duration, line) in lines.into_iter() {
        let padding = 2 + max_line_len - line.len();
        write!(result, "{}", line).unwrap();
        for _ in 0..padding {
            write!(result, ".").unwrap();
        }
        if line.contains("Stop") {
            writeln!(result, "[{}] <-- working", duration).unwrap();
        } else {
            writeln!(result, "[{}]", duration).unwrap();
        }
    }

    writeln!(result, "").unwrap();
    writeln!(result, "(+/-) Increase/decrease preferred work time").unwrap();

    write!(
        result,
        "\nPlease select what you want to do by pressing numbers (1-9) or (x): ",
    )
    .unwrap();

    result
}

fn mandatory_break_time_for_working_time(working_time: TimeDuration) -> TimeDuration {
    if working_time.minutes - (9 * 60) > 0 {
        TimeDuration { minutes: 45 }
    } else if working_time.minutes - (6 * 60) > 0 {
        TimeDuration { minutes: 30 }
    } else {
        TimeDuration::zero()
    }
}
