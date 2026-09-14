use std::collections::HashMap;
use std::sync::{
    Arc, atomic::{AtomicBool, AtomicU32},
};
use arc_swap::ArcSwap;
use crossterm::event::KeyCode;

use crate::geometry::Point;
use crate::math::rangef;
use crate::parsing::TokenNode;
use super::{
    AudioEngine,
    KeyResult, 
    PaneController, 
    PlotMode,
    PlotView2d,
};


/**
 * 
 */
pub struct WaveformController {
    pub waveform: Waveform,
    pub playback: PlaybackState,
    pub view: PlotView2d,
    pub points: Vec<Point>,
    pub audio: AudioEngine,
}

impl WaveformController {
    // -------------------------
    // Initialize WaveformController along with
    // the AudioEngine and the shared frequency atomic
    // -------------------------
    pub fn new(waveform: Waveform) -> Result<Self, String> {
        let playback = PlaybackState::default();

        let frequency_atomic = Arc::new(AtomicU32::new(
            (playback.frequency as f32).to_bits(),
        ));
        let volume_atomic = Arc::new(AtomicU32::new(
            playback.volume.to_bits()
        ));
        let playing_atomic = Arc::new(AtomicBool::new(false));

        let amp_callback = waveform.amplitude_callback();

        let audio = AudioEngine::new(
            move |phase| amp_callback(phase),
            frequency_atomic,
            volume_atomic,
            playing_atomic,
        )?;

        let mut controller = Self {
            waveform,
            playback,
            view: PlotView2d::default(),
            points: Vec::new(),
            audio,
        };

        // Set viewport to show two waveform cycles
        controller.view.viewport.x_min = 0.0;
        controller.view.viewport.x_max =
            2.0 / controller.playback.frequency;

        controller.view.viewport.y_min = -1.2;
        controller.view.viewport.y_max = 1.2;

        controller.refresh_points(0.0);

        Ok(controller)
    }

    pub fn refresh_points(&mut self, playback_phase: f64) {
        let count = self.view.samples.max(2);
        let time_span = self.view.viewport.x_max - self.view.viewport.x_min;

        self.points.clear();

        for i in 0..count {
            // Get x-point's cycle time offset in seconds
            let fraction = i as f64 / (count - 1) as f64;   // fraction in cycle [0, 1]
            let time_offset = fraction * time_span;

            // phase: cycle position [0, 1]
            // playback_phase: current cycle position for playback [0, 1]
            //
            // freq (cycles / sec) * time_offset (sec) = cycles
            // - How much cycle phase does the time offset push you
            let phase = (
                playback_phase 
                + self.playback.frequency * time_offset
            ).rem_euclid(1.0);  // loop around % 1.0 positive modulo

            let amplitude = self.waveform.sample_at_phase(phase) * self.playback.volume;

            self.points.push(Point::new(
                self.view.viewport.x_min + time_offset,
                amplitude as f64,
            ));
        }
    }

    pub fn set_frequency(&self) {
        self.audio.set_frequency(self.playback.frequency as f32);
    }

    pub fn set_volume(&mut self, volume: f32) {
        if !volume.is_finite() {
            return;
        }

        self.playback.volume = volume.clamp(0.0, 1.0);
        self.audio.set_volume(self.playback.volume);
    }
}

impl PaneController for WaveformController {
    /**
     * Render 2D waveform points to the terminal
     */
    fn render(&self, target: &mut crate::rendering::Buffer) {
        self.view.renderer.render(
            &self.points,
            &self.view.viewport,
            target,
        );
    }

    /**
     * 
     */
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> KeyResult {
        match key.code {
            // -------------------------
            // Exit
            // -------------------------
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('x') => {
                self.playback.playing = false;
                self.audio.set_playing(false);

                KeyResult::Exit
            },

            // -------------------------
            // Pause / Resume
            // -------------------------
            KeyCode::Char(' ') => {
                let playing = self.playback.playing;

                self.playback.playing = !playing;
                self.audio.set_playing(!playing);

                KeyResult::Changed
            }

            // -------------------------
            // Zoom in/out cycle range
            // -------------------------
            // Zoom out
            KeyCode::Char('q') => {
                self.waveform.set_cycle_range(
                    self.waveform.cycle_start - 0.1, 
                    self.waveform.cycle_end + 0.1,
                );
                

                KeyResult::Changed
            }
            // Zoom in
            KeyCode::Char('e') => {
                self.waveform.set_cycle_range(
                    self.waveform.cycle_start + 0.1, 
                    self.waveform.cycle_end - 0.1,
                );

                KeyResult::Changed
            }

            // -------------------------
            // Pan cycle range
            // -------------------------
            // Left
            KeyCode::Char('a') => {
                self.waveform.set_cycle_range(
                    self.waveform.cycle_start - 0.1, 
                    self.waveform.cycle_end - 0.1,
                );

                KeyResult::Changed
            }
            // Right
            KeyCode::Char('d') => {
                self.waveform.set_cycle_range(
                    self.waveform.cycle_start + 0.1, 
                    self.waveform.cycle_end + 0.1,
                );

                KeyResult::Changed
            }

            // -------------------------
            // Adjust frequency
            // -------------------------
            KeyCode::Right => {
                self.playback.frequency += 10.0;
                self.set_frequency();

                KeyResult::Changed
            }
            KeyCode::Left => {
                self.playback.frequency -= 10.0;
                self.set_frequency();

                KeyResult::Changed
            }

            // -------------------------
            // Adjust volume
            // -------------------------
            KeyCode::Down => {
                self.set_volume(self.playback.volume - 0.05);
                KeyResult::Changed
            }

            KeyCode::Up => {
                self.set_volume(self.playback.volume + 0.05);
                KeyResult::Changed
            }

            _ => KeyResult::Ignored,
        }
    }

    /**
     * Read audio engine's recent published phase,
     * refresh waveform points and request a redraw
     */
    fn update(&mut self, _delta_s: f64) -> bool {
        self.set_frequency();

        let phase = self.audio.phase();
        self.refresh_points(phase as f64);

        true
    }

    /**
     * Config text block for rendering in Pane
     */
    fn config_text(&self) -> Vec<String> {
        vec![
            format!(
                "Time window    ({:.2}, {:.2}) ms",
                self.view.viewport.x_min * 1000.0, 
                self.view.viewport.x_max * 1000.0,
            ),
            format!(
                "Amplitude      ({:.2}, {:.2})",
                self.view.viewport.y_min, self.view.viewport.y_max,
            ),
            format!(
                "Cycle domain   ({:.2}, {:.2})",
                self.waveform.cycle_start, 
                self.waveform.cycle_end,
            ),
            format!(
                "Resolution     {}", self.waveform.resolution(),
            ),
            format!(
                "Frequency      {:.0} Hz", self.playback.frequency,
            ),
            format!(
                "Volume         {:.0}%", self.playback.volume * 100.0,
            ),
            format!(
                "Period         {:.2} ms",
                self.view.viewport.x_max - self.view.viewport.x_min 
                / 2.0 * 1000.0,
            ),
        ]
    }

    /**
     * Tag text for top-right label
     */
    fn tag_text(&self) -> String {
        format!(
            "[{:.2}, {:.2}) · {} Hz · {:.0}% vol.", 
            self.waveform.cycle_start,
            self.waveform.cycle_end,
            self.playback.frequency, 
            self.playback.volume * 100.0,
        )
    }

    /**
     * Label text for top-left label next to number
     */  
    fn label_text(&self) -> String {
        "2D Audio".to_string()
    }
}


/**
 * Describes the 2D audio signal being played and displayed,
 * its expression, cycle range, and waveform table
 */
pub struct Waveform {
    pub expression: TokenNode,
    pub cycle_start: f64,       // start x-value of one cycle
    pub cycle_end: f64,         // end x-value of one cycle
    pub table: Vec<f32>,        // y-values within the cycle
    audio_table: Arc<ArcSwap<Vec<f32>>>,    // shared table with audio callback
}

impl Waveform {
    /**
     * Build Waveform from expression AST
     */
    pub fn build_waveform(
        expression: &str,
        cycle_start: f64,
        cycle_end: f64,
        resolution: usize,
    ) -> Result<Self, String> {
        // Construct AST and validate that it only uses X
        let tree = TokenNode::new(expression)?;
        tree.validate_variables(PlotMode::TwoD, false)?;

        // Build Waveform with empty table
        let mut waveform = Self {
            expression: tree,
            cycle_start,
            cycle_end,
            table: Vec::new(),
            audio_table: Arc::new(ArcSwap::from_pointee(Vec::<f32>::new())),
        };

        // Rebuild table with given resolution
        waveform.build_table(resolution);

        Ok(waveform)
    }

    // -------------------------
    // Amplitude callback to serve values
    // to the AudioEngine given its passed phase values
    // - Clones the waveform table once, the closure owns the copy
    // -------------------------
    pub fn amplitude_callback(
        &self,
    ) -> impl Fn(f32) -> f32 + Send + 'static + use<> {
        let shared_table = Arc::clone(&self.audio_table);

        move |phase: f32| {
            let table = shared_table.load();

            if table.is_empty() || !phase.is_finite() {
                return 0.0;
            }

            let position =
                (phase as f64).rem_euclid(1.0) * table.len() as f64;

            let index = position.floor() as usize;
            let fraction = (position - index as f64) as f32;

            let a = table[index];
            let b = table[(index + 1) % table.len()];

            a + (b - a) * fraction
        }
    }

    /**
     * Get y-value (amplitude) at given phase in cycle [0, 1],
     * interpolating between neighboring x-indices
     */
    pub fn sample_at_phase(&self, phase: f64) -> f32 {
        if self.table.is_empty() || !phase.is_finite() {
            return 0.0;
        }

        // position: actual fraction of point in cycle
        let position = phase.rem_euclid(1.0) * self.table.len() as f64;
        // index: point's closest floor index to its position
        let index = position.floor() as usize;
        // fraction: distance between neighboring indices for interpolation 
        let fraction = (position - index as f64) as f32;

        let a = self.table[index];
        let b = self.table[(index + 1) % self.table.len()];

        // Interpolate between floor index value and 
        // the true position between neighboring indices
        a + (b - a) * fraction
    }

    pub fn build_table(&mut self, resolution: usize) {
        // Rebuild the table at given resolution
        self.table = Vec::new();
        let mut vars = HashMap::from([
            ("x".to_string(), 0.0)
        ]);
            
        let x_vals = rangef(
            self.cycle_start as f64, 
            self.cycle_end as f64, 
            resolution,
            false,      // don't include duplicate cycle endpoint
        );

        for x in x_vals {
            *vars.get_mut("x").unwrap() = x;
            let y = self.expression.evaluate(&vars) as f32;

            self.table.push(if y.is_finite() { y } else { 0.0 });
        }

        // Store shared atomic table
        self.audio_table.store(Arc::new(self.table.clone()));
    }

    // Number of samples per cycle
    pub fn resolution(&self) -> usize {
        self.table.len()
    }

    pub fn set_resolution(&mut self, resolution: usize) {
        self.build_table(resolution);
    }

    pub fn set_expression(&mut self, expression: TokenNode) {
        let resolution = self.resolution();

        self.expression = expression;
        self.build_table(resolution);
    }

    pub fn set_cycle_range(&mut self, cycle_start: f64, cycle_end: f64) {
        let resolution = self.resolution();

        self.cycle_start = cycle_start;
        self.cycle_end = cycle_end;
        self.build_table(resolution);
    }
}


/**
 * Holds audio playback state for playing/paused,
 * playback position, loop boundaries, volume
 */
pub struct PlaybackState {
    pub frequency: f64,         // cycles / second
    pub volume: f32,
    pub playing: bool,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            frequency: 440.0,
            volume: 1.0,
            playing: false,
        }
    }
}