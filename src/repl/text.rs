pub const TOP_LEFT: char = '┌';
pub const TOP_RIGHT: char = '┐';
pub const BOTTOM_LEFT: char = '└';
pub const BOTTOM_RIGHT: char = '┘';

pub const TOP_LEFT_RD: char = '╭';
pub const TOP_RIGHT_RD: char = '╮';
pub const BOTTOM_LEFT_RD: char = '╰';
pub const BOTTOM_RIGHT_RD: char = '╯';

pub const HORIZONTAL: char = '─';
pub const VERTICAL: char = '│';


pub const HELP_2D: &str =
    "[↑↓←→/wasd] Pan │ [e/q] Zoom in/out │ [%/\"] Split pane | [ijkl] Resize pane";

pub const HELP_3D: &str =
    "[↑↓←→/wasd] Orbit │ [e/q] Zoom in/out │ [r] Reset camera │ [%/\"] Split pane | [ijkl] Resize pane";

pub const HELP_WAVEFORM: &str =
    "[→/←] Frequency up/down │ [↑/↓] Volume up/down │ [space] Play/pause │ [%/\"] Split pane | [ijkl] Resize pane";

pub const HELP_COMMANDS: &str=
    r#"
[Ctrl]+[c]                              Exit termview       
[c] | config                            Toggle settings panel
[h] | help                              Show help
[x]                                     Exit pane focus
────────────────────────────────────────────────────────────────────────
p | plot <expression>                   Plot using active dimension
p(2,3) | plot(2,3) <expression>         Plot in 2D or 3D
r | replot                              Replot last expression

proj <p,o>                              Set 3D projection method
                                        (Perspective, Orthographic)
view <x1> <x2> <y1> <y2>                Set visible coordinate range
view <x1> <x2> <y1> <y2> <z1> <z2>
view <x,y,z> <min> <max>                Set one axis range
samples <count>                         Set number of sampled points
show axes [0,1]                         Toggle axes visibility
show ticks [0,1]                        Toggle ticks / labels visibility
────────────────────────────────────────────────────────────────────────
a | animate <expression>                Plot animation
                                        - Expr. must use time (t)
a(2,3) | animate(2,3) <expression>      Animate in 2D or 3D

speed <value>                           Set animation speed
time <value>                            
────────────────────────────────────────────────────────────────────────
play <expression>                       Play 2D plot as waveform
"#;