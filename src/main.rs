use std::{error::Error, fs, str};

use err_derive::Error;

#[derive(Debug, Error)]
#[error(display = "invalid diff: {}", _0)]
struct ParseError(String);

enum Op {
    Add(f32),
    Sub(f32),
}

impl TryFrom<&str> for Op {
    type Error = Box<dyn Error>;
    fn try_from(s: &str) -> Result<Op, Self::Error> {
        let (sign, rest) = s.split_at(1);
        if rest.is_empty() {
            Err(ParseError("missing sign".to_string()))?;
        }
        let delta = str::parse(rest)?;
        match sign {
            "-" => Ok(Op::Sub(delta)),
            "+" => Ok(Op::Add(delta)),
            _ => Err(ParseError(s.to_string()))?,
        }
    }
}

fn run(diff: &str) -> Result<(), Box<dyn Error>> {
    const MAX: &'static str = "/sys/class/backlight/intel_backlight/max_brightness";
    const CUR: &'static str = "/sys/class/backlight/intel_backlight/brightness";

    let read_to_f32 = |path: &str| -> Result<f32, Box<dyn Error>> {
        Ok(str::parse(fs::read_to_string(path)?.trim())?)
    };

    let max = read_to_f32(MAX)?;
    let cur = read_to_f32(CUR)?;

    let percent = cur / max;
    println!("was: {}% ({})", (percent * 100.0).round(), cur);

    let new = match Op::try_from(diff)? {
        Op::Add(delta) => {
            let n = percent + (delta / 100.0);
            if n > 100.0 {
                max
            } else {
                n * max
            }
        }
        Op::Sub(delta) => {
            let n = percent - (delta / 100.0);
            if n < 0.0 {
                0.0
            } else {
                n * max
            }
        }
    };

    let rnew = new.round();
    println!("setting to: {}% ({})", ((new / max) * 100.0).round(), rnew);
    fs::write(CUR, &format!("{}", rnew))?;
    Ok(())
}

fn print_help(cmd: &str) {
    println!("{}: change laptop backlight brightness by percentage", cmd);
    println!("{}: <diff>", cmd);
    println!("    diff: [+|-]<0-100>");
}

fn main() {
    let mut args = std::env::args();
    let cmd = args.next().unwrap();
    let diff = match args.next() {
        Some(d) => {
            if d == "--help" {
                print_help(&cmd);
                return;
            } else {
                d
            }
        }
        None => {
            println!("missing diff");
            println!("usage: {} <diff>", cmd);
            println!("use --help for more info");
            std::process::exit(1);
        }
    };

    if let Err(e) = run(&diff) {
        println!("{}", e);
    }
}
