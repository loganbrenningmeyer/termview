use crate::rendering::{Projection, PerspectiveProjection, OrthographicProjection};

pub enum Command {
    SetDimension(u8),
    SetView {
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        z_bounds: Option<(f64, f64)>,
    },
    SetProjection(Projection),
    SetSamples(usize),
    ShowAxes(bool),
    ShowTicks(bool),
    ShowBorder(bool),
    Plot(String),
    Plot3d(String),
    Replot,
    Config,
    Help,
    Quit,
}

impl Command {
    pub fn parse(line: &str) -> Result<Command, String> {
        // Split args by whitespace
        let arguments: Vec<&str> = line.split_whitespace().collect();

        // Map to proper parser
        match arguments.as_slice() {
            ["q"] | ["Q"] | ["quit"] | ["exit"] => Ok(Command::Quit),
            ["h"] | ["H"] | ["help"] => Ok(Command::Help),
            ["c"] | ["C"] | ["config"] | ["cfg"] => Ok(Command::Config),
            ["replot"] => Ok(Command::Replot),
            
            ["set", "view", args @ ..] => Self::parse_set_view(args),
            ["set", "proj", val] => Self::parse_set_projection(val),
            ["set", "dim", val] => Self::parse_set_dimension(val),
            ["set", "samples", val] => Self::parse_set_samples(val),

            ["show", args @ ..] => Self::parse_show(args),
            ["plot", args @ ..] => Self::parse_plot(args),
            ["plot3d", args @ ..] => Self::parse_plot3d(args),
            
            [] => Err("Empty command".into()),
            _ => Err(format!("Unknown command: {line}")),
        }
    }

    // -------------------------
    // termview> plot <function>
    // -------------------------
    fn parse_plot(args: &[&str]) -> Result<Command, String> {
        // Remove whitespace / combine into String
        let function_str = args.join("");
        Ok(Command::Plot(function_str))
    }

    // -------------------------
    // termview> plot3d <function of x and y>
    // -------------------------
    fn parse_plot3d(args: &[&str]) -> Result<Command, String> {
        let function_str = args.join("");

        if function_str.is_empty() {
            return Err("plot3d requires an expression".into());
        }

        Ok(Command::Plot3d(function_str))
    }

    // -------------------------
    // termview> show axes <0, 1>
    // termview> show border <0, 1>
    // termview> show ticks <0, 1>
    // -------------------------
    fn parse_show(args: &[&str]) -> Result<Command, String> {
        // Validate / extract two arguments
        let (arg, val) = match args {
            [arg, val] => (*arg, *val),
            _ => {
                return Err(format!(
                    r#"Expected one of:

        termview> show axes <0, 1>
        termview> show border <0, 1>
        termview> show ticks <0, 1>"#
                )
                .into());
            }
        };

        // Enum tuple variants can be used as functions. Selecting the
        // constructor here both validates the element and avoids matching
        // the same string a second time below.
        let command: fn(bool) -> Command = match arg {
            "axes" => Command::ShowAxes,
            "border" => Command::ShowBorder,
            "ticks" => Command::ShowTicks,
            _ => return Err(format!("Unknown command: show {arg}")),
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

        Ok(command(show))
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
    // -------------------------
    fn parse_set_view(args: &[&str]) -> Result<Command, String> {
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
    termview> set view (x_min, x_max) (y_min, y_max) (z_min, z_max)"#
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
