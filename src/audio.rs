use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use rand::{thread_rng, Rng};
use rand_distr::Uniform;
use rodio::{Decoder, OutputStream, Sink};
use rodio::source::Source;
use tokio::sync::mpsc::Receiver;


static DING_AUDIO_FILE_PATH: &str = "audio/elevator-ding.wav";

#[derive(Debug)]
pub enum AudioPlayerCommand {
    Play,
    Pause,
}

// Note that this function starts the audio player thread, but does not begin
// playback until a `Play` command is sent through the channel.
pub fn start_audio_player(mut cmd_channel: Receiver<AudioPlayerCommand>) {
    let (_stream, stream_handle) = OutputStream::try_default()
        .expect("to be able to use the default output device or fallback");
    let sink = Sink::try_new(&stream_handle)
        .expect("to be able to create a new sound sink for the created audio stream");
    
    while let Some(cmd) = cmd_channel.blocking_recv() {
        use AudioPlayerCommand::*;
        match cmd {
            Play => {
                if sink.is_paused() {
                    sink.play();
                    continue;
                }
                
                // If the sink is neither paused nor empty, it should already
                // be playing. Otherwise, adding the source should start it.
                if sink.empty() {
                    sink.append(BrownNoiseSource::new());
                }
            },
            Pause => {
                sink.pause();    
            },
        }
    }

    // The channel sent `None`: time to shut down
    sink.stop()
}

pub fn play_ding() {
    tokio::spawn(async {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        let audio_file = BufReader::new(File::open(DING_AUDIO_FILE_PATH).unwrap());
        let source = Decoder::new(audio_file).unwrap();

        let audio_duration = if let Some(float_millis) = source.total_duration() {
            float_millis.as_millis() as u64
        } else {
            3000 // Fall back to 3s in case the file does not provide a total duration
        };

        stream_handle.play_raw(source.convert_samples()).unwrap();


        std::thread::sleep(Duration::from_millis(audio_duration));
    });
}

struct BrownNoiseSource {
    sample_source: Vec<f32>,
    sample_index: usize,
}

impl BrownNoiseSource {
    pub fn new() -> Self {
        let freq = 44100;
        let duration_seconds = 2;
        let mut sample_source = Vec::with_capacity(freq);
        let mut rng = thread_rng();
        let mut prior_value = 0.0;
        let uniform_dist = Uniform::new(-1.0, 1.0);
        
        // We create `freq` number of samples for each second we want the 
        // output to last.
        for _ in 0..freq * duration_seconds {
            // This calculation is based on the one here:
            // https://noisehack.com/generate-noise-web-audio-api/
            //
            // TODO: improve this with additional filters / transforms to make
            // the noise less harsh.
            let mut val = rng.sample(uniform_dist);
            val = (prior_value + 0.02 * val) / 1.02;
            prior_value = val;
            val *= 3.5; // approximate gain compensation
            sample_source.push(val);
        }

        BrownNoiseSource {
            sample_source,
            sample_index: 0,
        }
    }
}

impl Iterator for BrownNoiseSource {
    type Item = f32;

    // TODO: adjust with a volume control
    fn next(&mut self) -> Option<f32> {
        let item = self.sample_source[self.sample_index];
        self.sample_index = (self.sample_index + 1) % self.sample_source.len();

        Some(item)
    }
}

impl Source for BrownNoiseSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs(1))
    }
}
