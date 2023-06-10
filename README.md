# Calio

**Calio** is a tiny CLI tool that helps to visualize iCal file in the terminal.

## Installation

You can either install it via `cargo` or download the binaries from GitHub releases.

If you go the `cargo` route, you need to have it installed (usually using [rustup](https://rustup.rs)). In a terminal, run this command to install `calio`:

```
cargo install calio
```

Then you'll be able to run `calio` from whichever directory you're in.

If you want other installation option, please go to the [Release Page](https://github.com/oscarmcm/calio/releases/)


## How-To

**Calio** is easy to use, just provide a file path or stdin to read the
ics contents:

```
calio /some/file/path/cal.ics
cat /some/file/path.cal.ics | calio
```

And also comes with the following options:

```
--keep-alive  Keep the app running and do not exit on stdout.
-h, --help    Print help information.
```

Run with `--help/-h` for detailed usage.
