# Thyme 

Thyme is a simple little console application used for day-to-day timetracking. It supports
custom project names and generates daily reports. All data is written to / read from 
human readable textfiles for easy viewing and editing. 

## Program interface:
```
Hello! Today is Sunday 28. Feb (28.02.2021)
You started today at 09:21
You are working since 11:56 [00:27 (0.45h)]
                                                                @@@@@@
=================================================             @@      @@
Total work duration:         01:11 (1.18h) (100%)            @   @ @    @
  - Project activities:      00:40 (0.67h) ( 56%)           @            @
  - Non-Project activities:  00:31 (0.52h) ( 44%)            @@@@@@@@@@@@
Total break duration:        01:51 (1.85h)


=================================================
(1) Take a break
(2) Stop [Watch online videos]
(3) Begin ["Bugfixing"]
(4) Begin [Play with the cats]
(5) Begin [Workout (go to the fridge)]
(6) Begin [Build a sandcastle]
(7) Begin [Other important thing]

Please select what you want to do with numbers (1-9):

```


# Usage

Just place `thyme.exe` into a directory where it has write access to (preferably an empty directory).
Launching it will create a `database` folder and a `project_names.txt` file where Thyme stores and
reads its information.

All files can be opened and edited with a simple text editor. This can be useful for time 
corrections (`database/{your_date}.txt`) or adding/removing project names (`project_names.txt`).

**Example stamp events file (`database/{your_date}.txt`):**
```
09:21 - Begin [Work (Non-Project)]
09:33 - Begin [Bugfixing]
09:35 - Begin [Build a sandcastle]
09:42 - Begin [Play with the cats]
09:43 - Leave
10:41 - Begin [Work (Non-Project)]
11:00 - Leave
11:10 - Begin [Workout]
11:11 - Leave
11:16 - Begin [Play with the cats]
11:18 - Leave
11:56 - Begin [Watch online videos]
```

**Example project list file (`project_names.txt`):**
```
Watch online videos
Bugfixing
Play with the cats
Workout
Build a sandcastle
Other important thing
```

A daily report will be automatically generated and live updated to `today_report.txt` 
every minute (with a copy to `database/{your_date}__report.txt`) while Thyme is running.

**Example generated report file (`today__report.txt`):**

```
Report for Sunday 28. Feb (28.02.2021)


Activity Durations:
=====================

00:31 (0.52h) - Work (Non-Project)
00:27 (0.45h) - Watch online videos
00:07 (0.12h) - Build a sandcastle
00:03 (0.05h) - Play with the cats
00:02 (0.03h) - "Bugfixing"
00:01 (0.02h) - Workout (go to the fridge)

-------------

Total work duration:         01:11 (1.18h) (100%)
  - Project activities:      00:40 (0.67h) ( 56%)
  - Non-Project activities:  00:31 (0.52h) ( 44%)
Total break duration:        01:51 (1.85h)



Detailed Activity List:
=========================

09:21 - 09:33 [00:12 (0.20h)] - [Work (Non-Project)]
09:33 - 09:35 [00:02 (0.03h)] - ["Bugfixing"]
09:35 - 09:42 [00:07 (0.12h)] - [Build a sandcastle]
09:42 - 09:43 [00:01 (0.02h)] - [Play with the cats]
09:43 - 10:41 [00:58 (0.97h)] - [Break]
10:41 - 11:00 [00:19 (0.32h)] - [Work (Non-Project)]
11:00 - 11:10 [00:10 (0.17h)] - [Break]
11:10 - 11:11 [00:01 (0.02h)] - [Workout (go to the fridge)]
11:11 - 11:16 [00:05 (0.08h)] - [Break]
11:16 - 11:18 [00:02 (0.03h)] - [Play with the cats]
11:18 - 11:56 [00:38 (0.63h)] - [Break]
11:56 - <now> [00:27 (0.45h)] - [Watch online videos]
```



# Building it

Assuming we have [Git](https://git-scm.com/) installed first we need to clone and initialize this 
repository via:

```
git clone https://github.com/kerskuchen/thyme.git --recursive
```

Assuming we have [Rust](https://www.rust-lang.org/) installed and can run `cargo` commands we can 
build a release version by just running 

```
cargo run --package ct_executable_packager
```
This creates a new folder named `windows_shipping` which contains the final executable ready to run 
with all needed resources.

If we have the [Resource Hacker](http://angusj.com/resourcehacker/) tool in our `%PATH` the 
above command script will also set a launcher icon and version information for our 
executable.

# Development

We can build a debug version by running the usual `cargo build` command. The 
[Rust](https://www.rust-lang.org/) website has good information about how to start development 
with Rust.

For development it is a good idea to check out the `cottontail` submodule on the master branch via

```
cd cottontail
git checkout master
```

That will make sure that we don't accidentally commit something to `cottontail` in the 
detached `HEAD` state.
