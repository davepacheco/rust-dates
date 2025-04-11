// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{Context, bail};
use chrono::{
    DateTime, Duration, Local, NaiveDate, SecondsFormat, TimeZone, Utc,
};

const USAGE: &str = r#"
usage: dates              # prints current time (in several forms)
       dates TIME         # prints time TIME (in several forms)
       dates [+-]DELTA    # prints current time offset by DELTA
       dates T1 T2        # prints T1, T2, and the delta between them
       dates T1 [+-]DELTA # prints T1, DELTA, and T2 = T1 + DELTA
"#;

fn main() {
    if let Err(error) = doit() {
        eprintln!("dates: {:#}", error);
        eprintln!("{USAGE}");
        std::process::exit(2);
    }
}

fn doit() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();

    match args.len() {
        0 => print_time("now", Utc::now()),
        1 => handle_one(&args[0])?,
        2 => handle_two(&args[0], &args[1])?,
        _ => {
            bail!("too many arguments");
        }
    }

    Ok(())
}

fn handle_one(arg: &str) -> anyhow::Result<()> {
    if let Ok(delta) = parse_delta(arg) {
        let now = Utc::now();
        let then = now + delta;
        print_time("time 1", now);
        print_delta("delta", delta);
        print_time("time 2", then);
    } else if let Ok(time) = parse_time(arg) {
        print_time("time", time);
    } else {
        bail!("Could not parse {arg:?} as either a time or a delta");
    }
    Ok(())
}

fn handle_two(a: &str, b: &str) -> anyhow::Result<()> {
    let t1 =
        parse_time(a).with_context(|| format!("parsing {a:?} as a time"))?;
    if let Ok(t2) = parse_time(b) {
        print_time("time 1", t1);
        print_time("time 2", t2);
        print_delta("delta", t2 - t1);
    } else if let Ok(d) = parse_delta(b) {
        let t2 = t1 + d;
        print_time("time 1", t1);
        print_delta("delta", d);
        print_time("time 2", t2);
    } else {
        bail!("Could not parse {b:?} as either a time or a delta");
    }
    Ok(())
}

fn parse_time(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    if let Ok(ts) = s.parse::<i64>() {
        // treat as milliseconds since epoch
        return Ok(Utc.timestamp_millis_opt(ts).unwrap());
    }
    if let Ok(ts) = s.parse::<f64>() {
        // treat as seconds.fractional
        let millis = (ts * 1000.0).round() as i64;
        return Ok(Utc.timestamp_millis_opt(millis).unwrap());
    }

    // Try parsing with chrono
    let dt = DateTime::parse_from_rfc3339(s)
        .or_else(|_| DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3f%z"))
        .map(|n| n.to_utc())
        .or_else(|_| {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(|n| n.and_hms_opt(0, 0, 0).unwrap().and_utc())
        })?;
    Ok(dt.with_timezone(&Utc))
}

fn parse_delta(s: &str) -> Result<Duration, ()> {
    let (sign, rest) = match s.chars().next() {
        Some('+') => (1, &s[1..]),
        Some('-') => (-1, &s[1..]),
        _ => return Err(()),
    };

    let unit = rest
        .chars()
        .rev()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    let value_str = &rest[..rest.len() - unit.len()];
    let value: f64 = value_str.parse().map_err(|_| ())?;

    let seconds = match unit.as_str() {
        "ms" => value / 1000.0,
        "s" => value,
        "m" => value * 60.0,
        "h" => value * 3600.0,
        "d" => value * 86400.0,
        _ => return Err(()),
    };

    Ok(Duration::milliseconds((sign as f64 * seconds * 1000.0) as i64))
}

fn print_time(label: &str, dt: DateTime<Utc>) {
    let timestamp = dt.timestamp() as f64
        + (dt.timestamp_subsec_micros() as f64 / 1_000_000.0);
    println!(
        "{:<8} {:>20.6} s = {}",
        label,
        timestamp,
        dt.with_timezone(&Local).to_rfc3339_opts(SecondsFormat::Micros, true),
    );
    println!(
        "         {:>20.6}   = {}",
        "",
        dt.to_rfc3339_opts(SecondsFormat::Micros, true)
    );
}

fn print_delta(label: &str, delta: Duration) {
    let total_micros = delta.num_microseconds().unwrap_or(0);
    let total_secs = total_micros as f64 / 1_000_000.0;

    let sign = if total_micros < 0 { "-" } else { " " };
    let abs_us = total_micros.abs();
    let secs = abs_us / 1_000_000;
    let micros = abs_us % 1_000_000;

    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;

    println!(
        "{:<8} {:>20.6} s = {}{}d {:02}h {:02}m {:02}.{:06}s",
        label, total_secs, sign, d, h, m, s, micros
    );
}
