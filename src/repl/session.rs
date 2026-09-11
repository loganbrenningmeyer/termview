use crate::{
    geometry::{Mesh, sample_surface_at, sample_curve_at}, 
    parsing::{TokenNode}, 
    rendering::{
        PlotViewport2d, 
        PlotViewport3d,
    }, 
    ui::{
        Animation, 
        ContentController,
        LastPlot,
        PlotContent, 
        PlotMode,
        PlotController, 
        PlotView2d,
        PlotView3d,
        CurvePlot, 
        SurfacePlot,
        Waveform,
        WaveformController,
    },
};
use super::Command;


pub enum SessionOutput {
    None,
    Message(String),
    Redraw,
    PaneUpdated { title: String },
}


pub struct Session;

impl Session {
    /**
     * Replace current PlotController's content with the newly
     * built CurvePlot or SurfacePlot built from the given
     * expression string, PlotMode, and animation mode
     */
    fn replace_plot(
        &mut self,
        plot_controller: &mut PlotController,
        expression: String,
        mode: PlotMode,
        animated: bool,
    ) -> Result<SessionOutput, String> {
        let (content, title) = match mode {
            PlotMode::TwoD => {
                let curve = self.build_curve(
                    &plot_controller.view_2d, 
                    &expression, 
                    animated,
                )?;

                (PlotContent::TwoD(curve), format!(" y = {expression} "))
            }
            PlotMode::ThreeD => {
                let surface = self.build_surface(
                    &plot_controller.view_3d,
                    &expression,
                    animated,
                )?;

                (PlotContent::ThreeD(surface), format!(" z = {expression} "))
            }
        };

        plot_controller.content = content;
        plot_controller.last_plot = Some(LastPlot { 
            expression,
            mode,
            animated,
        });

        Ok(SessionOutput::PaneUpdated { title })
    }

    /**
     * Tokenizes & parses expression, validates its variable
     * identifier nodes (x, t if animated), and computes y = f(x) samples to
     * build a CurvePlot for PlotContent
     */
    fn build_curve(
        &self,
        view_2d: &PlotView2d,
        expression: &str,
        animated: bool,
    ) -> Result<CurvePlot, String> {
        // Tokenize / parse expression into AST 
        let tree = TokenNode::new(expression)?;

        // Validate variable identifiers
        tree.validate_variables(PlotMode::TwoD, animated)?;

        // Perform initial sample with t = 0
        // - No effect if not animated
        let mut points = Vec::with_capacity(view_2d.samples);

        sample_curve_at(
            &tree,
            view_2d.viewport.x_min,
            view_2d.viewport.x_max,
            view_2d.samples,
            0.0,
            &mut points,
        ); 

        // Define animation parameters if enabled
        let animation = if animated {
            Some(Animation {
                time: 0.0,
                speed: 1.0,
                period: 5.0,
                phase: 0.0,
                playing: true,
            }) 
        } else {
            None
        };

        // Build 2D CurvePlot
        let curve = CurvePlot {
            expression: tree,
            points,
            animation,
        };

        Ok(curve)
    }

    /**
     * Tokenizes & parses expression, validates its variable
     * identifier nodes (x, y, t if animated), and computes 
     * z = f(x, y) samples to build a SurfacePlot for PlotContent
     */
    fn build_surface(
        &self,
        view_3d: &PlotView3d,
        expression: &str,
        animated: bool,
    ) -> Result<SurfacePlot, String> {
        // Tokenize / parse expression into AST 
        let tree = TokenNode::new(expression)?;

        // Validate variable identifiers
        tree.validate_variables(PlotMode::ThreeD, animated)?;

        // Create placehold Mesh grid to compute samples
        // - Placeholder function z = 0, ensuring all finite vertices initially
        let viewport = view_3d.viewport;
        let (x_samples, y_samples) = view_3d.samples;

        let mut mesh = Mesh::surface(
            viewport.x_min,
            viewport.x_max,
            viewport.y_min,
            viewport.y_max,
            x_samples,
            y_samples,
            |_: f64, _: f64| { 0.0 },
        );

        sample_surface_at(&tree, 0.0, &mut mesh);

        // Define animation parameters if enabled
        let animation = if animated {
            Some(Animation {
                time: 0.0,
                speed: 1.0,
                period: 5.0,
                phase: 0.0,
                playing: true,
            }) 
        } else {
            None
        };

        // Build 3D SurfacePlot
        let surface = SurfacePlot {
            expression: tree,
            mesh,
            animation,
        };

        Ok(surface)
    }

    /**
     * Build WaveformController Pane, replacing the current 
     * ContentController Pane and retaining its size
     */
    fn open_waveform(
        &self, 
        controller: &mut ContentController,
        expression: &str
    ) -> Result<SessionOutput, String> {
        // Build Waveform with default values
        let waveform = Waveform::build_waveform(
            expression, 
            0.0, 
            std::f64::consts::TAU, 
            2048,
        )?;

        // Construct WaveformController
        let waveform_controller = WaveformController::new(waveform)?;

        // Replace the current pane's controller
        *controller = ContentController::Waveform(waveform_controller);

        Ok(SessionOutput::PaneUpdated { 
            title: format!(" y = {expression} "),
        })
    }

    pub fn execute(
        &mut self, 
        controller: &mut ContentController,
        command: Command
    ) -> Result<SessionOutput, String> {
        match command {
            // -------------------------
            // Audio playback
            // - Replace pane with waveform playback
            // -------------------------
            Command::Play(expression) => {
                self.open_waveform(controller, &expression)
            }

            // -------------------------
            // General commands
            // -------------------------
            Command::Help => {
                Ok(SessionOutput::Message(Self::help_text()))
            }
            Command::Quit => Ok(SessionOutput::None),

            // -------------------------
            // 2D / 3D Plot
            // - Replace pane with plot
            // -------------------------
            command @ (
                Command::Plot(_)
                | Command::Plot2d(_)
                | Command::Plot3d(_)
                | Command::Animate(_)
                | Command::Animate2d(_)
                | Command::Animate3d(_)
            ) => {
                // Already Plot pane, preserve view and settings
                if let ContentController::Plot(plot) = controller {
                    self.execute_plot(plot, command)
                // Waveform pane, build Plot pane and replace waveform
                } else {
                    let mut plot = PlotController::default();
                    let output = self.execute_plot(&mut plot, command)?;

                    *controller = ContentController::Plot(plot);

                    Ok(output) 
                }
            }

            // -------------------------
            // Commands acting on the current feature
            // - Don't disrupt and replace the Pane
            // -------------------------
            command => match controller {
                ContentController::Plot(plot) => {
                    self.execute_plot(plot, command)
                }
                ContentController::Waveform(waveform) => {
                    self.execute_waveform(waveform, command)
                }
            },
        }
    }

    fn execute_waveform(
        &mut self,
        controller: &mut WaveformController,
        command: Command,
    ) -> Result<SessionOutput, String> {
        match command {
            Command::Pause => {
                controller.playback.playing = false;
                Ok(SessionOutput::Redraw)
            }

            Command::Resume => {
                controller.playback.playing = true;
                Ok(SessionOutput::Redraw)
            }

            Command::ShowAxes(show) => {
                controller.view.renderer.show_axes = show;
                Ok(SessionOutput::Redraw)
            }

            Command::Config => {
                Ok(SessionOutput::Message(format!(
                    "Frequency: {} Hz\nVolume: {}\nPlaying: {}",
                    controller.playback.frequency,
                    controller.playback.volume,
                    controller.playback.playing,
                )))
            }

            _ => Err("This command is not supported by waveform panes".into()),
        }
    }

    fn execute_plot(
        &mut self,
        plot_controller: &mut PlotController,
        command: Command,
    ) -> Result<SessionOutput, String> {
        match command {
            // -------------------------
            // Plot / Plot2d / Plot3d / Replot
            // -------------------------
            Command::Plot(expression) => {
                self.replace_plot(
                    plot_controller,
                    expression,
                    plot_controller.active_plot_mode,
                    false,
                )
            }

            Command::Plot2d(expression) => {
                let output = self.replace_plot(
                    plot_controller, 
                    expression, 
                    PlotMode::TwoD, 
                    false,
                )?;

                plot_controller.active_plot_mode = PlotMode::TwoD;
                Ok(output)
            }

            Command::Plot3d(expression) => {
                let output = self.replace_plot(
                    plot_controller,
                    expression,
                    PlotMode::ThreeD,
                    false,
                )?;

                plot_controller.active_plot_mode = PlotMode::ThreeD;
                Ok(output)
            }

            Command::Replot => {
                let Some(last) = plot_controller.last_plot.clone() else {
                    return Ok(SessionOutput::Message(
                        "This pane has no previous plot".into(),
                    ));
                };

                self.replace_plot(
                    plot_controller,
                    last.expression,
                    last.mode,
                    last.animated,
                )
            }

            // -------------------------
            // Animate / Animate3d
            // -------------------------
            Command::Animate(expression) => {
                self.replace_plot(
                    plot_controller,
                    expression,
                    plot_controller.active_plot_mode,
                    true,
                )
            }

            Command::Animate2d(expression) => {
                let output = self.replace_plot(
                    plot_controller,
                    expression,
                    PlotMode::TwoD,
                    true,
                )?;

                plot_controller.active_plot_mode = PlotMode::TwoD;
                Ok(output)
            }

            Command::Animate3d(expression) => {
                let output = self.replace_plot(
                    plot_controller,
                    expression,
                    PlotMode::ThreeD,
                    true,
                )?;

                plot_controller.active_plot_mode = PlotMode::ThreeD;
                Ok(output)
            }

            // -------------------------
            // Animation Pause / Resume / Speed & Time
            // -------------------------
            command @ (
                Command::Pause 
                | Command::Resume
                | Command::SetSpeed(_)
                | Command::SetTime(_)
            ) => {
                let animation = plot_controller
                    .content
                    .animation_mut()
                    .ok_or_else(|| "The current plot has no animation".to_string())?;

                match command {
                    Command::Pause => animation.playing = false,
                    Command::Resume => animation.playing = true,
                    
                    Command::SetSpeed(speed) => {
                        if !speed.is_finite() {
                            return Err("Animation speed must be finite".into());
                        }
                        
                        animation.speed = speed;
                    }
                    
                    Command::SetTime(time) => {
                        if !time.is_finite() {
                            return Err("Animation time must be finite".into())
                        }
                        animation.time = time;
                        animation.phase = time;
                        
                        // Time elapsed, resample animation
                        plot_controller.resample();
                    }

                    _ => unreachable!()
                }

                Ok(SessionOutput::Redraw)
            }

            Command::SetDimension(dimension) => {
                plot_controller.active_plot_mode = match dimension {
                    2 => PlotMode::TwoD,
                    3 => PlotMode::ThreeD,
                    _ => return Ok(SessionOutput::Message("plot dimension must be 2 or 3".into())),
                };

                Ok(SessionOutput::None)
            }

            Command::SetView {
                x_min,
                x_max,
                y_min,
                y_max,
                z_bounds,
            } => {
                if x_min >= x_max {
                    return Ok(SessionOutput::Message("x_min must be less than x_max".into()));
                }

                if y_min >= y_max {
                    return Ok(SessionOutput::Message("y_min must be less than y_max".into()));
                }

                match plot_controller.active_plot_mode {
                    PlotMode::TwoD => {
                        if z_bounds.is_some() {
                            return Ok(SessionOutput::Message("2D view expects x and y bounds only".into()));
                        }

                        plot_controller.view_2d.viewport = PlotViewport2d {
                            x_min,
                            x_max,
                            y_min,
                            y_max,
                        };
                    }
                    PlotMode::ThreeD => {
                        let Some((z_min, z_max)) = z_bounds else {
                            return Ok(SessionOutput::Message("3D view also requires z_min and z_max".into()));
                        };

                        if z_min >= z_max {
                            return Ok(SessionOutput::Message("z_min must be less than z_max".into()));
                        }

                        plot_controller.view_3d.viewport = PlotViewport3d {
                            x_min,
                            x_max,
                            y_min,
                            y_max,
                            z_min,
                            z_max,
                        };
                    }
                }

                Ok(SessionOutput::None)
            }

            Command::SetProjection(proj) => {
                plot_controller.view_3d.camera.projection = proj;

                Ok(SessionOutput::None)
            }

            Command::SetSamples(samples) => {
                if samples < 2 {
                    return Ok(SessionOutput::Message("samples must be at least 2".into()));
                }

                match plot_controller.active_plot_mode {
                    PlotMode::TwoD => plot_controller.view_2d.samples = samples,
                    PlotMode::ThreeD => plot_controller.view_3d.samples = (samples, samples),
                }
                Ok(SessionOutput::None)
            }

            Command::ShowAxes(show) => {
                match plot_controller.active_plot_mode {
                    PlotMode::TwoD => plot_controller.view_2d.renderer.show_axes = show,
                    PlotMode::ThreeD => plot_controller.view_2d.renderer.show_axes = show,
                }
                Ok(SessionOutput::None)
            }
            
            Command::ShowTicks(show) => {
                match plot_controller.active_plot_mode {
                    PlotMode::TwoD => plot_controller.view_2d.renderer.show_ticks = show,
                    PlotMode::ThreeD => {
                        plot_controller.view_3d.renderer.axes_renderer.show_ticks = show;
                        plot_controller.view_3d.renderer.axes_renderer.show_labels = show;
                    }
                }
                Ok(SessionOutput::None)
            }

            Command::ShowBorder(show) => {
                match plot_controller.active_plot_mode {
                    PlotMode::TwoD => plot_controller.view_2d.renderer.show_border = show,
                    PlotMode::ThreeD => plot_controller.view_3d.renderer.show_border = show,
                }
                Ok(SessionOutput::None)
            }

            Command::Config => {
                Ok(SessionOutput::Message(
                    self.config_text(plot_controller)
                ))
            }

            Command::Help => {
                Ok(SessionOutput::Message(
                    Self::help_text()
                ))
            }

            Command::Quit => {
                Ok(SessionOutput::None)
            }

            _ => Ok(SessionOutput::None)
        }
    }

    /**
     * Print current PlotViewport / PlotRenderer configuration
     */
    fn config_text(&self, plot_controller: &PlotController) -> String {
        let viewport_2d = &plot_controller.view_2d.viewport;
        let renderer_2d = &plot_controller.view_2d.renderer;
        let samples_2d = plot_controller.view_2d.samples;

        let viewport_3d = &plot_controller.view_3d.viewport;
        let renderer_3d = &plot_controller.view_3d.renderer;
        let samples_3d = plot_controller.view_3d.samples;
        let camera_3d = &plot_controller.view_3d.camera;

        let active_settings = match plot_controller.active_plot_mode {
            PlotMode::TwoD => format!(
            r#"
Current settings
  Dimension          2D
  View               ({}, {}) to ({}, {})
  Projection (3D)    {}
  Samples            {}
  Axes               {}
  Ticks              {}
  Border             {}
"#,
            viewport_2d.x_min,
            viewport_2d.y_min,
            viewport_2d.x_max,
            viewport_2d.y_max,
            camera_3d.projection,
            samples_2d,
            on_off(renderer_2d.show_axes),
            on_off(renderer_2d.show_ticks),
            on_off(renderer_2d.show_border),
            ),
            PlotMode::ThreeD => format!(
            r#"
Current settings
  Dimension          3D
  View               ({}, {}, {}) to ({}, {}, {})
  Projection (3D)    {}
  Samples            {} x {}
  Axes               {}
  Ticks              {}
  Border             {}
"#,
                viewport_3d.x_min,
                viewport_3d.y_min,
                viewport_3d.z_min,
                viewport_3d.x_max,
                viewport_3d.y_max,
                viewport_3d.z_max,
                camera_3d.projection,
                samples_3d.0,
                samples_3d.1,
                on_off(renderer_3d.show_axes),
                on_off(renderer_3d.axes_renderer.show_ticks),
                on_off(renderer_3d.show_border),
            ),
        };

        active_settings
    }

    /**
     * Print list of available commands
     */
    fn help_text() -> String {
        return 
            r#"
==============================
      Termview commands 
==============================

  plot <expression>                             Plot using the active dimension
  plot3d <expression>                           Plot a function of x and y, and switch to 3D
  replot                                        Plot the last expression again

  set dim <2|3>                                 Set the active plot dimension

  set view <xmin> <xmax> <ymin> <ymax>          Set the visible coordinate range
  set view (<xmin>, <xmax>) (<ymin>, <ymax>)
  set view <xmin> <xmax> <ymin> <ymax> <zmin> <zmax>

  set proj <p|o>                                Set the 3D projection method 
  set proj <perspective|orthographic>

  set samples <count>                           Set the number of sampled points

  show axes <true|false|1|0>                    Show or hide the axes
  show ticks <true|false|1|0>                   Show or hide ticks and labels
  show border <true|false|1|0>                  Show or hide the plot border
  
  [C] | cfg | config                            Display the current plotting configuration
  [H] | help                                    Display this help

  [Q] | quit | exit                             Exit termview

==============================
        Plot Controls 
==============================

  [↑] [←] [↓] [→]                               Rotate camera [UP] [DOWN] [LEFT] [RIGHT]
  [W] [A] [S] [D] 

  [Q] [E]                                       Zoom [in] [out]

  [R]                                           Reset camera view

  [X] | [Esc] | [Enter]                         Exit plot                                     

=========================
"#.to_string();
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}
