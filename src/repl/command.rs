use crate::math::{
    Projection, 
    PerspectiveProjection, 
    OrthographicProjection,
};

pub enum Command {
    SetDimension(u8),
    SetView {
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        z_bounds: Option<(f64, f64)>,
    },
    SetViewAxis {
        axis: char,
        min: f64,
        max: f64,
    },
    SetProjection(Projection),
    SetSamples(usize),
    SetCycle {
        start: f64,
        end: f64,
    },

    ShowAxes(Option<bool>),     // None toggles
    ShowTicks(Option<bool>),

    Plot(String),
    Plot2d(String),
    Plot3d(String),
    Replot,

    Play(String),

    Animate(String),
    Animate2d(String),
    Animate3d(String),
    Pause,
    Resume,
    SetSpeed(f64),
    SetTime(f64),
    
    Config,
    Help,
    Quit,
}

impl Command {
    pub fn parse(line: &str) -> Result<Command, String> {
        // Handle `y=...` or `z=...` plot commands before splitting whitespace
        let line = line.trim().to_lowercase();

        if let Some((left, right)) = line.split_once('=') {
            let variable = left.trim();

            if matches!(variable, "y" | "z") {
                let expression = right.trim();

                if expression.is_empty() {
                    return Err(format!(
                        "Expected an expression after {variable} ="
                    ));
                }

                let args: Vec<&str> = expression.split_whitespace().collect();

                return match variable {
                    "y" => Self::parse_plot(&args),
                    "z" => Self::parse_plot3d(&args),
                    _ => unreachable!(),
                };
            }
        }

        // Split args by whitespace
        let arguments: Vec<&str> = line.split_whitespace().collect();

        // Map to proper parser
        match arguments.as_slice() {
            ["q"] | ["quit"] | ["exit"]  => Ok(Command::Quit),
            ["h"] | ["help"]             => Ok(Command::Help),
            ["c"] | ["config"] | ["cfg"] => Ok(Command::Config),
            ["r"] | ["replot"]           => Ok(Command::Replot),
            
            ["view", args @ ..] | ["set", "view", args @ ..] 
                => Self::parse_set_view(args),
            ["proj", val] | ["set", "proj", val] 
                => Self::parse_set_projection(val),
            ["dim", val] | ["set", "dim", val] 
                => Self::parse_set_dimension(val),
            ["s", val] | ["samples", val] | ["set", "samples", val] 
                => Self::parse_set_samples(val),
            ["show", args @ ..]   => Self::parse_show(args),

            ["p", args @ ..]  | ["plot", args @ ..]   => Self::parse_plot(args),
            ["p2", args @ ..] | ["plot2", args @ ..] => Self::parse_plot2d(args),
            ["p3", args @ ..] | ["plot3", args @ ..] => Self::parse_plot3d(args),

            ["a", args @ ..]  | ["animate", args @ ..]   => Self::parse_animate(args),
            ["a2", args @ ..] | ["animate2", args @ ..] => Self::parse_animate2d(args),
            ["a3", args @ ..] | ["animate3", args @ ..] => Self::parse_animate3d(args),

            ["play", args @ ..] => Self::parse_play(args),
            ["cycle", args @ ..] | ["set", "cycle", args @ ..] => {
                Self::parse_set_cycle(args)
            }

            ["pause"]  => Ok(Command::Pause),
            ["resume"] => Ok(Command::Resume),

            ["speed", val] | ["set", "speed", val] => Self::parse_set_speed(val),
            ["time", val] | ["set", "time", val]   => Self::parse_set_time(val),
            
            [] => Err("Empty command".into()),
            _ => Err(format!("Unknown command: {line}")),
        }
    }

    // -------------------------
    // termview> cycle <start> <end>
    // termview> set cycle <start> <end>
    // -------------------------
    fn parse_set_cycle(args: &[&str]) -> Result<Command, String> {
        let [start, end] = args else {
            return Err("Usage: cycle <start> <end>".into());
        };

        let start = start.parse::<f64>()
            .map_err(|_| format!("Invalid cycle start: {start}"))?;

        let end = end.parse::<f64>()
            .map_err(|_| format!("Invalid cycle end: {end}"))?;

        if !start.is_finite()
            || !end.is_finite()
            || end <= start
            || !(end - start).is_finite()
        {
            return Err(
                "Cycle bounds must be finite, with end greater than start \
                and a finite span.".into()
            );
        }

        Ok(Command::SetCycle { start, end })
    }

    // -------------------------
    // termview> play <function>
    // -------------------------
    fn parse_play(args: &[&str]) -> Result<Command, String> {
        let function_str = args.join("");
        Ok(Command::Play(function_str))
    }

    // -------------------------
    // termview> plot <function>
    // -------------------------
    fn parse_plot(args: &[&str]) -> Result<Command, String> {
        let function_str = args.join("");
        Ok(Command::Plot(function_str))
    }

    // -------------------------
    // termview> plot2d <function>
    // -------------------------
    fn parse_plot2d(args: &[&str]) -> Result<Command, String> {
        let function_str = args.join("");
        Ok(Command::Plot2d(function_str))
    }

    // -------------------------
    // termview> plot3d <function of x and y>
    // -------------------------
    fn parse_plot3d(args: &[&str]) -> Result<Command, String> {
        let function_str = args.join("");
        Ok(Command::Plot3d(function_str))
    }

    // -------------------------
    // termview> animate <function>
    // -------------------------
    fn parse_animate(args: &[&str]) -> Result<Command, String> {
        // Remove whitespace / combine into String
        let function_str = args.join("");
        Ok(Command::Animate(function_str))
    }

    // -------------------------
    // termview> animate2d <function>
    // -------------------------
    fn parse_animate2d(args: &[&str]) -> Result<Command, String> {
        // Remove whitespace / combine into String
        let function_str = args.join("");
        Ok(Command::Animate2d(function_str))
    }

    // -------------------------
    // termview> animate3d <function>
    // -------------------------
    fn parse_animate3d(args: &[&str]) -> Result<Command, String> {
        // Remove whitespace / combine into String
        let function_str = args.join("");
        Ok(Command::Animate3d(function_str))
    }

    // -------------------------
    // termview> set speed <val>
    // -------------------------
    fn parse_set_speed(val: &str) -> Result<Command, String> {
        let speed = val.parse::<f64>()
                             .map_err(|_| format!("Invalid animation speed: {val}"))?;
        Ok(Command::SetSpeed(speed))
    }

    // -------------------------
    // termview> set time <val>
    // -------------------------
    fn parse_set_time(val: &str) -> Result<Command, String> {
        let time = val.parse::<f64>()
                             .map_err(|_| format!("Invalid animation time (t): {val}"))?;
        Ok(Command::SetTime(time))
    }

    // -------------------------
    // termview> show axes [0, 1]
    // termview> show ticks [0, 1]
    // - No value toggles the current setting
    // -------------------------
    fn parse_show(args: &[&str]) -> Result<Command, String> {
        // Validate / extract element and optional value
        let (arg, val) = match args {
            [arg] => (*arg, None),
            [arg, val] => (*arg, Some(*val)),
            _ => {
                return Err(format!(
                    r#"Expected one of:

        termview> show axes [0, 1]
        termview> show ticks [0, 1]"#
                )
                .into());
            }
        };

        // Enum tuple variants can be used as functions. Selecting the
        // constructor here both validates the element and avoids matching
        // the same string a second time below.
        let command: fn(Option<bool>) -> Command = match arg {
            "axes" => Command::ShowAxes,
            "ticks" => Command::ShowTicks,
            _ => return Err(format!("Unknown command: show {arg}")),
        };

        // No value: toggle
        let Some(val) = val else {
            return Ok(command(None));
        };

        let show = match val.to_lowercase().as_str() {
            "1" | "true" => true,
            "0" | "false" => false,
            _ => {
                return Err(format!(
                    r#"Expected one of:

        termview> show {arg} <0, 1>
        termview> show {arg} <false, true>"#
                )
                .into());
            }
        };

        Ok(command(Some(show)))
    }

    // -------------------------
    // termview> set samples <n>
    // -------------------------
    fn parse_set_samples(val: &str) -> Result<Command, String> {
        let num_samples = val.parse::<usize>()
                             .map_err(|_| format!("Invalid number of samples: {val}"))?;

        Ok(Command::SetSamples(num_samples))
    }

    // -------------------------
    // termview> set dim <2, 3>
    // -------------------------
    fn parse_set_dimension(val: &str) -> Result<Command, String> {
        match val {
            "2" => Ok(Command::SetDimension(2)),
            "3" => Ok(Command::SetDimension(3)),
            _ => Err("plot dimension must be 2 or 3".into()),
        }
    }

    // -------------------------
    // termview> set view <x_min x_max y_min y_max>
    // termview> set view <(x_min, x_max) (y_min, y_max)>
    // termview> set view <x, y, z> <min max>
    // -------------------------
    fn parse_set_view(args: &[&str]) -> Result<Command, String> {
        // Single axis: view <x, y, z> <min max>
        if let [axis @ ("x" | "y" | "z"), rest @ ..] = args {
            return Self::parse_set_view_axis(axis, rest);
        }

        // Turn punctuation into whitespace.
        let normalized = args.join(" ")
                             .replace(['(', ')', ','], " ");

        let values: Vec<f64> = normalized
            .split_whitespace()
            .map(|value| {
                value
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid number: {value}"))
            })
            .collect::<Result<Vec<f64>, String>>()?;

        let (x_min, x_max, y_min, y_max, z_bounds) = match values.as_slice() {
            [x_min, x_max, y_min, y_max] => {
                (*x_min, *x_max, *y_min, *y_max, None)
            }
            [x_min, x_max, y_min, y_max, z_min, z_max] => {
                (*x_min, *x_max, *y_min, *y_max, Some((*z_min, *z_max)))
            }
            _ => {
                return Err(
                    r#"Expected one of:

    termview> set view x_min x_max y_min y_max
    termview> set view x_min x_max y_min y_max z_min z_max

    termview> set view (x_min, x_max) (y_min, y_max)
    termview> set view (x_min, x_max) (y_min, y_max) (z_min, z_max)

    termview> set view <x, y, z> min max"#
                        .into(),
                );
            }
        };

        Ok(Command::SetView {
            x_min,
            x_max,
            y_min,
            y_max,
            z_bounds,
        })
    }

    // -------------------------
    // termview> set view <x, y, z> <min max>
    // termview> set view <x, y, z> <(min, max)>
    // -------------------------
    fn parse_set_view_axis(axis: &str, args: &[&str]) -> Result<Command, String> {
        // Turn punctuation into whitespace.
        let normalized = args.join(" ")
                             .replace(['(', ')', ','], " ");

        let [min, max] = normalized.split_whitespace().collect::<Vec<_>>()[..] else {
            return Err(format!("Usage: view {axis} <min> <max>"));
        };

        let min = min.parse::<f64>()
            .map_err(|_| format!("Invalid {axis}_min: {min}"))?;

        let max = max.parse::<f64>()
            .map_err(|_| format!("Invalid {axis}_max: {max}"))?;

        Ok(Command::SetViewAxis {
            axis: axis.chars().next().unwrap(),
            min,
            max,
        })
    }

    // -------------------------
    // termview> set proj <p, o>
    // termview> set proj <perspective, orthographic>
    // -------------------------
    fn parse_set_projection(val: &str) -> Result<Command, String> {
        match val {
            "p" | "perspective" => Ok(Command::SetProjection(Projection::Perspective(PerspectiveProjection::default()))),
            "o" | "orthorgraphic" => Ok(Command::SetProjection(Projection::Orthographic(OrthographicProjection::default()))),
            _ => Err("projection must be perspective (p) or orthographic (o)".into()),
        }
    }
}
