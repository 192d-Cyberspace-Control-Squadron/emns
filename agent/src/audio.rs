use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct AudioPlayer {
    sounds_dir: PathBuf,
}

impl AudioPlayer {
    pub fn new(sounds_dir: PathBuf) -> Self {
        Self { sounds_dir }
    }

    /// Play a sound file by name
    pub fn play_sound(&self, filename: &str) -> Result<()> {
        let sound_path: PathBuf = self.sounds_dir.join(filename);

        if !sound_path.exists() {
            log::warn!(
                "Sound file not found: {}, using system beep",
                sound_path.display()
            );
            self.play_system_beep();
            return Ok(());
        }

        log::info!("Playing sound: {}", sound_path.display());

        // Create an output stream (this needs to stay alive during playback)
        let (_stream, stream_handle) =
            OutputStream::try_default().context("Failed to get default audio output stream")?;

        // Create a sink to play audio
        let sink = Sink::try_new(&stream_handle).context("Failed to create audio sink")?;

        // Load the audio file
        let file: File = File::open(&sound_path)
            .with_context(|| format!("Failed to open sound file: {}", sound_path.display()))?;
        let source: Decoder<BufReader<File>> = Decoder::new(BufReader::new(file))
            .with_context(|| format!("Failed to decode audio file: {}", sound_path.display()))?;

        // Play the sound
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }

    /// Play a system beep as fallback
    fn play_system_beep(&self) {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{MessageBeep, MB_ICONEXCLAMATION};
            let _ = MessageBeep(MB_ICONEXCLAMATION);
        }
    }

    /// Play sound in a separate thread (non-blocking)
    pub fn play_sound_async(&self, filename: String) {
        let sounds_dir: PathBuf = self.sounds_dir.clone();
        std::thread::spawn(move || {
            let player: AudioPlayer = AudioPlayer::new(sounds_dir);
            if let Err(e) = player.play_sound(&filename) {
                log::error!("Failed to play sound {}: {}", filename, e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_beep() {
        let player: AudioPlayer = AudioPlayer::new(PathBuf::from("./sounds"));
        player.play_system_beep();
    }
}
