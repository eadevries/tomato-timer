use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use rodio::{Decoder, OutputStream};
use rodio::source::Source;


static DING_AUDIO_FILE_PATH: &str = "audio/elevator-ding.wav";

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
