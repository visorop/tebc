use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::time::Duration;

use serde::Deserialize;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("in.txt"))]
    input_file: String,

    #[arg(short, long, default_value_t = String::from("out.txt"))]
    output_file: String,
}

#[derive(Debug, Deserialize)]
struct Beat {
    time_start: f64,
    _time_stop: f64,
    _b_type: char,
}

fn main() {
    let args = Args::parse();
    process(args.input_file.as_str(), args.output_file.as_str());
}

fn process(from_path: &str, to_path: &str) {
    let to_file = File::create(to_path).expect("Output File error: cannot create");
    let mut file = BufWriter::new(to_file);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_path(from_path)
        .expect("Input File error");

    rdr.deserialize::<Beat>()
        .enumerate()
        .map(|r| {
            (
                r.0,
                r.1.unwrap_or_else(|_| panic!("Cannot parse line {}", r.0)),
            )
        })
        .map(|beat| (beat.0, Duration::from_secs_f64(beat.1.time_start)))
        .map(|duration| (duration.0, duration.1.as_millis()))
        .map(|millis| {
            format!(
                "{} {}\n",
                millis_to_string(millis.1)
                    .unwrap_or_else(|| panic!("Illegal value at line {}", millis.0)),
                millis.0
            )
        })
        .for_each(|formatted_millis| {
            file.write_all(formatted_millis.as_bytes())
                .expect("Cannot write a result line to file");
        });

    file.flush().expect("Cannot flush output file");
}

fn millis_to_string(millis: u128) -> Option<String> {
    if !(1..=86400000).contains(&millis) {
        return None;
    }

    let hours = millis % 86400000 / 3600000;
    let minutes = millis % 3600000 / 60000;
    let seconds = millis % 60000 / 1000;
    let millis = millis % 1000;

    Some(format!(
        "{:0>2}:{:0>2}:{:0>2}.{:0>3}",
        hours, minutes, seconds, millis
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millis_to_string_convetrs_ok() {
        assert_eq!(millis_to_string(12).unwrap_or_default(), "00:00:00.012");
        assert_eq!(millis_to_string(123).unwrap_or_default(), "00:00:00.123");
        assert_eq!(
            millis_to_string((60 * 2 + 3) * 1000 + 123).unwrap_or_default(),
            "00:02:03.123"
        );
        assert_eq!(
            millis_to_string((3600 * 22 + 60 * 54 + 31) * 1000 + 60).unwrap_or_default(),
            "22:54:31.060"
        );
        assert_eq!(
            millis_to_string((86400 * 2 + 3600 * 3 + 60 * 41 + 11) * 1000 + 60).unwrap_or_default(),
            ""
        );
        assert_eq!(millis_to_string(0).unwrap_or_default(), "");
    }
}
