//! Cross-platform audio player module.
//!
//! This module provides audio playback functionality for different platforms.
//! It supports Windows, macOS, and Linux with appropriate audio players.

use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Playback state for audio player
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PlaybackState {
    /// No audio is playing
    #[default]
    Idle,
    /// Audio is currently playing
    Playing(String),
    #[allow(dead_code)]
    /// Playback completed successfully
    Completed,
    /// Playback failed with an error message
    Failed(String),
}

/// Audio player for playing WAV files
pub struct AudioPlayer {
    current_process: Arc<Mutex<Option<std::process::Child>>>,
    state: Arc<Mutex<PlaybackState>>,
}

impl AudioPlayer {
    /// Creates a new audio player
    pub fn new() -> Self {
        AudioPlayer {
            current_process: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(PlaybackState::Idle)),
        }
    }

    /// Gets the current playback state
    pub fn get_state(&self) -> PlaybackState {
        self.state.lock().expect("State mutex poisoned").clone()
    }

    /// Checks if audio is currently playing
    pub fn is_playing(&self) -> bool {
        matches!(self.get_state(), PlaybackState::Playing(_))
    }

    /// Updates playback state if playback has finished
    pub fn update_state_if_finished(&self) {
        if self.is_playing() {
            let mut process = self.current_process.lock().expect("Process mutex poisoned");
            if let Some(mut child) = process.take() {
                // Try to check if process has finished
                if let Ok(Some(_)) = child.try_wait() {
                    tracing::info!("Audio playback finished");
                    *self.state.lock().expect("State mutex poisoned") = PlaybackState::Idle;
                } else {
                    // Process still running, put it back
                    *process = Some(child);
                }
            }
        }
    }

    /// Stops the current playback if any
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Kill current process if running
        {
            let mut process = self.current_process.lock().expect("Process mutex poisoned");
            if let Some(mut child) = process.take() {
                #[cfg(unix)]
                {
                    use nix::sys::signal::{Signal, kill};
                    use nix::unistd::Pid;

                    // Try graceful shutdown first
                    let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);

                    // Wait a bit for graceful shutdown
                    std::thread::sleep(Duration::from_millis(100));

                    // Force kill if still running
                    if let Ok(None) = child.try_wait() {
                        let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGKILL);
                        let _ = child.wait();
                    }
                }

                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;

                    // Force kill on Windows
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }

        // Update state
        *self.state.lock().expect("State mutex poisoned") = PlaybackState::Idle;

        Ok(())
    }

    /// Plays the specified audio file
    pub fn play(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Check if file exists
        if !Path::new(file_path).exists() {
            return Err(format!("Audio file not found: {}", file_path).into());
        }

        // Stop any currently playing audio
        self.stop()?;

        // Start playback based on platform
        let child = self.play_audio(file_path)?;

        // Store process and update state
        {
            let mut process = self.current_process.lock().expect("Process mutex poisoned");
            *process = Some(child);
        }

        *self.state.lock().expect("State mutex poisoned") =
            PlaybackState::Playing(file_path.to_string());

        Ok(())
    }

    /// Waits for the current playback to complete
    #[allow(dead_code)]
    pub fn wait_for_completion(&self) -> Result<(), Box<dyn std::error::Error>> {
        let process_opt = self
            .current_process
            .lock()
            .expect("Process mutex poisoned")
            .take();

        if let Some(mut child) = process_opt {
            let status = child.wait()?;
            if status.success() {
                *self.state.lock().expect("State mutex poisoned") = PlaybackState::Completed;
            } else {
                *self.state.lock().expect("State mutex poisoned") =
                    PlaybackState::Failed("Playback process exited with error".to_string());
            }
        }

        Ok(())
    }

    /// Plays audio using platform-specific audio player
    fn play_audio(
        &self,
        file_path: &str,
    ) -> Result<std::process::Child, Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            self.play_windows(file_path)
        }

        #[cfg(target_os = "macos")]
        {
            self.play_macos(file_path)
        }

        #[cfg(not(any(windows, target_os = "macos")))]
        {
            self.play_linux(file_path)
        }
    }

    #[cfg(windows)]
    fn play_windows(
        &self,
        file_path: &str,
    ) -> Result<std::process::Child, Box<dyn std::error::Error>> {
        tracing::info!("Playing audio using Windows Media Player");

        // Convert path to Windows format if needed
        let windows_path = if file_path.contains('/') {
            file_path.replace('/', "\\")
        } else {
            file_path.to_string()
        };

        // Use PowerShell to play audio via Windows Media Player
        let powershell_script = format!(
            "$player = New-Object -ComObject WMPlayer.OCX;$player.URL = '{}';$player.controls.play();Start-Sleep -Seconds 1;while($player.playState -eq 3){{Start-Sleep -Seconds 1}}",
            windows_path
        );

        let child = Command::new("powershell")
            .args(&["-Command", &powershell_script])
            .spawn()?;

        Ok(child)
    }

    #[cfg(target_os = "macos")]
    fn play_macos(
        &self,
        file_path: &str,
    ) -> Result<std::process::Child, Box<dyn std::error::Error>> {
        let players = vec![
            ("afplay", vec![file_path]),
            ("ffplay", vec!["-nodisp", "-autoexit", file_path]),
        ];

        self.try_play_audio_players(players, file_path)
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    fn play_linux(
        &self,
        file_path: &str,
    ) -> Result<std::process::Child, Box<dyn std::error::Error>> {
        let players = vec![
            ("aplay", vec!["-q", file_path]),
            ("paplay", vec![file_path]),
            ("ffplay", vec!["-nodisp", "-autoexit", file_path]),
        ];

        self.try_play_audio_players(players, file_path)
    }

    #[cfg(any(target_os = "macos", not(any(windows, target_os = "macos"))))]
    fn try_play_audio_players(
        &self,
        players: Vec<(&str, Vec<&str>)>,
        _file_path: &str,
    ) -> Result<std::process::Child, Box<dyn std::error::Error>> {
        for (player, args) in players {
            if self.which_command(player).is_ok() {
                tracing::info!("Playing audio using: {}", player);

                let child = Command::new(player).args(&args).spawn()?;

                return Ok(child);
            }
        }

        Err("No audio player found. Please install aplay, paplay, or ffplay".into())
    }

    /// Checks if a command exists in PATH
    fn which_command(&self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("which").arg(command).output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!("Command '{}' not found", command).into())
        }
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_state_variants() {
        let state_idle = PlaybackState::Idle;
        let state_playing = PlaybackState::Playing("test.wav".to_string());
        let state_completed = PlaybackState::Completed;
        let state_failed = PlaybackState::Failed("error".to_string());

        assert_eq!(state_idle, PlaybackState::Idle);
        assert!(matches!(state_playing, PlaybackState::Playing(_)));
        assert_eq!(state_completed, PlaybackState::Completed);
        assert!(matches!(state_failed, PlaybackState::Failed(_)));
    }

    #[test]
    fn test_audio_player_creation() {
        let player = AudioPlayer::new();
        assert_eq!(player.get_state(), PlaybackState::Idle);
        assert!(!player.is_playing());
    }

    #[test]
    fn test_audio_player_default() {
        let player = AudioPlayer::default();
        assert_eq!(player.get_state(), PlaybackState::Idle);
    }
}
