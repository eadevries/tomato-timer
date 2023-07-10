use chrono::Duration;
use clap::{arg, command, value_parser};


static MAIN_HELP_TEXT: &str = include_str!("./help_text/main.txt");
static MUTED_HELP_TEXT: &str = include_str!("./help_text/muted.txt");
static START_HELP_TEXT: &str = include_str!("./help_text/start.txt");
static BROWN_NOISE_HELP_TEXT: &str = include_str!("./help_text/noise.txt");

static DEFAULT_SESSION_MILLIS: i64 = 1000 * 20;

#[derive(Debug)]
pub struct Options {
    pub start: bool,
    pub duration: Duration,
    pub muted: bool,
    pub noise: bool,
}

impl Options {
    pub fn new() -> Self {
        Options {
            start: true,
            duration: Duration::milliseconds(DEFAULT_SESSION_MILLIS),
            muted: false,
            noise: false,
        }
    }
}

pub fn get_cli_options() -> Options {
    let mut options = Options::new();

    let matches = command!()
        .arg(arg!(-d --duration <MINUTES> "Minutes for the timer to run")
             .value_parser(value_parser!(u32)))
        .arg(arg!(-s --start).help(START_HELP_TEXT))
        .arg(arg!(-M --muted).help(MUTED_HELP_TEXT))
        .arg(arg!(-n --"brown-noise").help(BROWN_NOISE_HELP_TEXT))
        .after_help(MAIN_HELP_TEXT)
        .get_matches();

    // Flags
    options.start = matches.get_flag("start");
    options.muted = matches.get_flag("muted");
    options.noise = matches.get_flag("brown-noise");

    // Other args
    if let Some(&minutes) = matches.get_one::<u32>("duration") {
        options.duration = Duration::minutes(minutes as i64);
    }

    options
}
