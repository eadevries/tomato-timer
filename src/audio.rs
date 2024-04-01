use std::io::{BufReader,Cursor};
use std::time::Duration;

use rodio::{Decoder, OutputStream};
use rodio::source::Source;


pub fn play_ding() {
    tokio::spawn(async {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        let audio_bytes = std::include_bytes!("../audio/elevator-ding.wav");
        let audio_buf = BufReader::new(Cursor::new(audio_bytes));
        let source = Decoder::new(audio_buf).unwrap();

        let audio_duration = if let Some(float_millis) = source.total_duration() {
            float_millis.as_millis() as u64
        } else {
            3000 // Fall back to 3s in case the file does not provide a total duration
        };

        stream_handle.play_raw(source.convert_samples()).unwrap();


        std::thread::sleep(Duration::from_millis(audio_duration));
    });
}
