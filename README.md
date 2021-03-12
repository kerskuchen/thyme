# Thyme 

Thyme is a simple little console application used for day-to-day timetracking. It supports
custom project names and generates daily reports. All data is written to / read from 
human readable textfiles for easy viewing and editing. 

## Program interface:
```
Today is Friday 12. Mar (12.03.2021) -- You started at 07:13
You are doing [Watch online videos] since 20:12 [00:08h]

=================================================

Total work duration:            11:28h (100%)
  - Activities (from list):     10:38h ( 93%)
  - Activities (non-specific):  00:50h (  7%)               @@@@@@
Total break duration:           01:39h                    @@      @@
                                                         @   @ @    @
                                                        @            @
=================================================        @@@@@@@@@@@@

(x) Take a break

<1> Stop  [Watch online videos].....[01:00h] <-- working
(2) Begin ["Bugfixing"].............[00:53h]
(3) Begin [Play with the cats]......[01:30h]
(4) Begin [Build a sandcastle]......[01:54h]
(5) Begin [Jump around].............[00:21h]
(6) Begin [Do the important thing]..[00:12h]
(7) Begin [Eat bananas].............[02:17h]
(8) Begin [Look out of the window]..[02:15h]
(9) Begin [Walk in the park]........[00:16h]

Please select what you want to do by pressing numbers (1-9) or (x):

```


# Usage

Just place `thyme.exe` into a directory where it has write access to (preferably an empty directory).
Launching it will create a `database` folder and a `project_names.txt` file where Thyme stores and
reads its information.

All files can be opened and edited with a simple text editor. This can be useful for time 
corrections (`database/{your_date}.txt`) or adding/removing project names (`project_names.txt`).

**Example stamp events file (`database/{your_date}.txt`):**
```
07:13 - Begin [Watch online videos]
08:05 - Begin ["Bugfixing"]
08:19 - Begin [Play with the cats]
08:56 - Begin [Look out of the window]
10:32 - Begin [Build a sandcastle]
11:03 - Begin [Jump around]
11:24 - Begin [Do the important thing]
11:36 - Leave
13:11 - Begin [Eat bananas]
15:28 - Begin [Look out of the window]
16:07 - Begin [Walk in the park]
16:23 - Begin [Work (Non-specific)]
17:13 - Begin [Play with the cats]
18:06 - Leave
18:10 - Begin [Build a sandcastle]
19:33 - Begin ["Bugfixing"]
20:12 - Begin [Watch online videos]
```

**Example project list file (`project_names.txt`):**
```
Watch online videos
"Bugfixing"
Play with the cats
Build a sandcastle
Jump around
Do the important thing
Eat bananas
Look out of the window
Walk in the park
```

A daily report will be automatically generated and live updated to `today_report.txt` 
every minute (with a copy to `database/{your_date}__report.txt`) while Thyme is running.

**Example generated report file (`today__report.txt`):**

```
Report for Friday 12. Mar (12.03.2021)


Activity Durations:
=====================

02:17h - Eat bananas
02:15h - Look out of the window
01:54h - Build a sandcastle
01:30h - Play with the cats
01:01h - Watch online videos
00:53h - "Bugfixing"
00:50h - Work (Non-specific)
00:21h - Jump around
00:16h - Walk in the park
00:12h - Do the important thing

-------------

Total work duration:            11:29h (100%)
  - Activities (from list):     10:39h ( 93%)
  - Activities (non-specific):  00:50h (  7%)
Total break duration:           01:39h



Detailed Activity List:
=========================

07:13 - 08:05 [00:52h] - [Watch online videos]
08:05 - 08:19 [00:14h] - ["Bugfixing"]
08:19 - 08:56 [00:37h] - [Play with the cats]
08:56 - 10:32 [01:36h] - [Look out of the window]
10:32 - 11:03 [00:31h] - [Build a sandcastle]
11:03 - 11:24 [00:21h] - [Jump around]
11:24 - 11:36 [00:12h] - [Do the important thing]
11:36 - 13:11 [01:35h] - [Break]
13:11 - 15:28 [02:17h] - [Eat bananas]
15:28 - 16:07 [00:39h] - [Look out of the window]
16:07 - 16:23 [00:16h] - [Walk in the park]
16:23 - 17:13 [00:50h] - [Work (Non-specific)]
17:13 - 18:06 [00:53h] - [Play with the cats]
18:06 - 18:10 [00:04h] - [Break]
18:10 - 19:33 [01:23h] - [Build a sandcastle]
19:33 - 20:12 [00:39h] - ["Bugfixing"]
20:12 - <now> [00:09h] - [Watch online videos]


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
