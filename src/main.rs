use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Utc};
use cursive::{Cursive, Rect, Vec2};
use cursive::reexports::crossbeam_channel::Sender as CBSender;
use cursive::reexports::enumset::EnumSet;
use cursive::theme::{Color, ColorStyle, Style, PaletteColor};
use cursive::utils::span::SpannedString;
use cursive::view::{Resizable, SizeConstraint, View};
use cursive::views::{FixedLayout, Layer, OnLayoutView, TextContent, TextView};
use tokio::sync::mpsc::{channel, Receiver, Sender};

mod cli;

#[derive(Debug, Clone)]
enum RunState {
    Unstarted,
    Running(RunningState),
    Paused(PausedState),
    Finished,
}

#[derive(Debug, Clone)]
struct RunningState {
    expiration: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct PausedState {
    duration_remaining: Duration,
}

struct TimerState {
    session_length: Duration,
    run_state: RunState,
}

impl TimerState {
    fn new(session_length: Duration) -> Self {
        TimerState {
            session_length,
            run_state: RunState::Unstarted,
        }
    }
}

#[tokio::main]
async fn main() {
    let options = cli::get_cli_options();

    let timer_state = Arc::new(Mutex::new(
        TimerState::new(options.duration)
    ));

    // Create the curses app which handles the TUI and watches for key inputs.
    let mut curses_app = cursive::default();

    curses_app.load_toml(include_str!("theme.toml")).unwrap();
    let bg_color = curses_app.current_theme().clone().palette[PaletteColor::Background];

    // Create styled text for the bottom info bar
    let hot_key_style = Style {
        effects: EnumSet::empty(),
        color: ColorStyle::front(Color::Rgb(200, 50, 200)),
    };
    // TODO: consider using manually indexed spans instead and extracting to 
    // a separate function.
    let mut info_bar_text = SpannedString::new();
    info_bar_text.append_plain(" (");
    info_bar_text.append_styled("s", hot_key_style);
    info_bar_text.append_plain(") start timer | (");
    info_bar_text.append_styled("p", hot_key_style);
    info_bar_text.append_plain(") pause toggle | (");
    info_bar_text.append_styled("q", hot_key_style);
    info_bar_text.append_plain(") quit");

    // Create an info bar at the bottom of the app to display controls
    curses_app.screen_mut().add_transparent_layer(
        OnLayoutView::new(
            FixedLayout::new().child(
                Rect::from_point(Vec2::zero()),
                Layer::new(TextView::new(info_bar_text))
                    .full_width(),
            ),
            |layout, size| {
                layout.set_child_position(
                    0,
                    Rect::from_size((0, size.y - 1), (size.x, 1)),
                );
                layout.layout(size);
            },
        ).full_screen(),
    );

    // Create a separate TextContent and add it to a TextView so we can keep a
    // reference to it and update it without retrieving the TextView from the
    // cursive app by name.
    let (h, m, s) = duration_to_hms(&options.duration);
    let timer_content = TextContent::new(format!("{:02}:{:02}:{:02}", h, m, s));
    curses_app.add_layer(TextView::new_with_content(timer_content.clone())
        .center()
        .resized(SizeConstraint::Fixed(12), SizeConstraint::Fixed(3)));

    curses_app.set_autorefresh(true);

    // Channel for sending messages to start / stop / pause / unpause the timer
    let (starter_send, mut starter_recv) = channel::<()>(16);
    // Channel on which to send time updates only
    let (time_send, time_recv) = channel::<(i64, i64, i64)>(16);
    // Channel to send callbacks to cursive (main thread) from other threads
    let to_main_thread = curses_app.cb_sink().clone();

    // Clone the references to the time update channel and main state mutex and 
    // pass them into a new thread which exists to wait for a signal from the
    // synchronous cursive app to start up a thread to send regular time 
    // updates and check for the session's ending.
    let timer_state_clone = timer_state.clone();
    let time_send_clone = time_send.clone();
    let to_main_thread_clone = to_main_thread.clone();
    tokio::spawn(async move {
        while let Some(()) = starter_recv.recv().await {
            send_updates(time_send_clone.clone(), timer_state_clone.clone(), &to_main_thread_clone).await;             
        }
    });

    // If the `start` flag was passed, immediately update the timer_state to
    // running and send the message which will start up the time update thread.
    if options.start {
        let mut ts = timer_state.lock().unwrap();
        let expiration = Utc::now() + options.duration;
        (*ts).run_state = RunState::Running(RunningState { expiration, });

        starter_send.send(()).await.unwrap();
    }

    // Spawn the thread to recv the time updates and update the view.
    let timer_content_clone = timer_content.clone();
    tokio::spawn(async move {
        recv_messages(time_recv, &timer_content_clone).await;
    });

    // Callbacks to handle user input
    let timer_state_clone = timer_state.clone();
    let starter_send_clone = starter_send.clone();
    curses_app.add_global_callback('p', move |_| {
        {
            let mut ts = timer_state_clone.lock().unwrap();
            let new_run_state = match &ts.run_state {
                RunState::Running(rs) => {
                    RunState::Paused(PausedState {
                        duration_remaining: rs.expiration - Utc::now(),
                    })
                },
                RunState::Paused(ps) => {
                    RunState::Running(RunningState {
                        expiration: Utc::now() + ps.duration_remaining,
                    })
                },
                _ => ts.run_state.clone(),
            };
            (*ts).run_state = new_run_state;
        }

        let starter_send_clone = starter_send_clone.clone();
        tokio::spawn(async move {
            starter_send_clone.clone().send(()).await.unwrap();
        });
    });

    curses_app.add_global_callback('q', |s| { s.quit() });

    let timer_state_clone = timer_state.clone();
    curses_app.add_global_callback('r', move |app| {
        let mut ts = timer_state_clone.lock().unwrap();
        if let RunState::Finished = ts.run_state {
            (*ts).run_state = RunState::Unstarted;

            let (h, m, s) = duration_to_hms(&options.duration);
            timer_content.set_content(format!("{:02}:{:02}:{:02}", h, m, s));

            // Set background color back to default
            let mut theme = app.current_theme().clone();
            theme.palette[PaletteColor::Background] = bg_color;
            app.set_theme(theme);
        }
    });

    let timer_state_clone = timer_state.clone();
    let starter_send_clone = starter_send.clone();
    curses_app.add_global_callback('s', move |_| { 
        {
            let mut ts = timer_state_clone.lock().unwrap();
            let new_run_state = match &ts.run_state {
                RunState::Unstarted => {
                    RunState::Running(RunningState {
                        expiration: Utc::now() + options.duration,
                    })
                },
                _ => ts.run_state.clone(),
            };
            (*ts).run_state = new_run_state;
        }

        let starter_send_clone = starter_send_clone.clone();
        tokio::spawn(async move {
            starter_send_clone.send(()).await.unwrap();
        });
    });
    

    // With the async threads spawned, we finish by running the synchronous
    // cursive app.
    curses_app.run();
}

async fn send_updates(
    send: Sender<(i64, i64, i64)>,
    timer_state: Arc<Mutex<TimerState>>,
    to_main_thread: &CBSender<Box<dyn FnOnce(&mut Cursive) + Send + 'static>>,
) {
    // To move into spawned thread, we cannot directly use the parameter, as it
    // will cannot outlast the function body.
    let to_main_thread = to_main_thread.clone();

    tokio::spawn(async move {
        let mut hours;
        let mut minutes;
        let mut seconds;
        let mut timer_finished;

        // Loop until the timer_state is no longer RunState::Running, sending
        // regular time updates.
        loop {
            { // additional block created to ensure that the mutex lock is 
              // dropped before the `await` when sending the time update.
                let mut ts = timer_state.lock().unwrap();
                if let RunState::Running(rs) = &ts.run_state {
                    let remaining_duration = rs.expiration - Utc::now();
                    timer_finished = remaining_duration.num_seconds() <= 0;
                    (hours, minutes, seconds) = duration_to_hms(&remaining_duration);
                } else {
                    // If we are not in the running state, do not send any
                    // updates and let the thread finish.
                    break;
                }
                
                // Two separate `timer_finished` checks so that we don't need
                // to reacquire the mutex after leaving the block.
                if timer_finished {
                    (hours, minutes, seconds) = (0, 0, 0);
                    (*ts).run_state = RunState::Finished;
                    to_main_thread.send(Box::new(|s| {
                        let mut theme = s.current_theme().clone();
                        theme.palette[PaletteColor::Background] = Color::Rgb(75, 0, 0);
                        s.set_theme(theme);
                    })).expect("to be able to send a callback to the main thread");
                }
            }

            // Send the updated time remaining so that the TUI can be updated.
            if let Err(_) = send.send((hours, minutes, seconds)).await {
                // If unable to send, the receiver has closed and we are
                // shutting down.
                break;
            }

            // If there is still time on the clock, sleep until the next 100ms
            // has elapsed.
            if !timer_finished {
                tokio::time::interval(Duration::milliseconds(100).to_std().unwrap());
            }
        }
    });
}

async fn recv_messages(mut receiver: Receiver<(i64, i64, i64)>, content: &TextContent) {
    while let Some((hours, minutes, seconds)) = receiver.recv().await {
        content.set_content(&format!("{:02}:{:02}:{:02}", hours, minutes, seconds));
    }
}

fn duration_to_hms(dur: &Duration) -> (i64, i64, i64) {
    let hours = dur.num_hours() % 100;
    let minutes = dur.num_minutes() % 60;
    let seconds = dur.num_seconds() % 60;

    (hours, minutes, seconds)
}
