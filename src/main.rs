use std::{error::Error, fs, str};

use clap::Parser;
use err_derive::Error;

#[derive(Debug, Parser)]
struct Args {
    #[clap(index = 1)]
    diff: String,
}

#[derive(Debug, Error)]
#[error(display = "invalid diff: {}", _0)]
struct ParseError(String);

enum Op {
    Add(u32),
    Sub(u32),
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

    let read_to_u32 = |path: &str| -> Result<u32, Box<dyn Error>> {
        Ok(str::parse(fs::read_to_string(path)?.trim())?)
    };

    let max = read_to_u32(MAX)?;
    let cur = read_to_u32(CUR)?;

    let new = match Op::try_from(diff)? {
        Op::Add(delta) => {
            let new = cur.saturating_add(delta);
            if new > max {
                max
            } else {
                new
            }
        }
        Op::Sub(delta) => cur.saturating_sub(delta),
    };

    fs::write(CUR, &format!("{}", new))?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args.diff) {
        println!("{}", e);
    }
}
