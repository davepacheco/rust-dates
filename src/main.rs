use chrono::{DateTime, Duration, FixedOffset, Local, TimeZone, Utc};
use clap::Parser;
use std::str::FromStr;

/// Parse and format timestamps, offsets, and deltas
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg()]
    values: Vec<String>,
}

fn main() {
    let args = Args::parse();

    match args.values.len() {
        0 => print_now(),
        1 => handle_single(&args.values[0]),
        2 => handle_two(&args.values[0], &args.values[1]),
        _ => eprintln!("Too many arguments"),
    }
}

fn print_now() {
    let now = Utc::now();
    print_time("now", now);
}

fn handle_single(arg: &str) {
    if let Ok(delta) = parse_delta(arg) {
        let now = Utc::now();
        let then = now + delta;
        print_time("time 1", now);
        print_delta("delta", delta);
        print_time("time 2", then);
    } else if let Ok(time) = parse_time(arg) {
        print_time("time", time);
    } else {
        eprintln!("Could not parse input: {}", arg);
    }
}

fn handle_two(a: &str, b: &str) {
    if let (Ok(t1), Ok(t2)) = (parse_time(a), parse_time(b)) {
        print_time("time 1", t1);
        print_time("time 2", t2);
        print_delta("delta", t2 - t1);
    } else if let (Ok(t1), Ok(d)) = (parse_time(a), parse_delta(b)) {
        let t2 = t1 + d;
        print_time("time 1", t1);
        print_delta("delta", d);
        print_time("time 2", t2);
    } else {
        eprintln!("Could not parse inputs: {} {}", a, b);
    }
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
        .or_else(|_| DateTime::parse_from_str(s, "%Y-%m-%d"))
        .or_else(|_| DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3f%z"))?;
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

// fn print_time(label: &str, dt: DateTime<Utc>) {
//     let timestamp = dt.timestamp() as f64 + (dt.timestamp_subsec_micros() as f64 / 1_000_000.0);
//     println!("{:<8} {:>14.3} s = {}", label, timestamp, dt.with_timezone(&Local));
//     println!("         {:>14.3} s = {}", timestamp, dt.to_rfc3339());
// }
// 
// fn print_delta(label: &str, delta: Duration) {
//     let secs = delta.num_seconds();
//     let ms = delta.num_milliseconds() - secs * 1000;
//     let sign = if delta < Duration::zero() { "-" } else { " " };
// 
//     let total_secs = delta.num_milliseconds() as f64 / 1000.0;
//     let d = secs / 86400;
//     let h = (secs % 86400) / 3600;
//     let m = (secs % 3600) / 60;
//     let s = secs % 60;
// 
//     println!(
//         "{:<8} {:>14.3} s = {}{}d {:02}h {:02}m {:02}.{:03}s",
//         label,
//         total_secs,
//         sign,
//         d.abs(),
//         h.abs(),
//         m.abs(),
//         s.abs(),
//         ms.abs()
//     );
// }

fn print_time(label: &str, dt: DateTime<Utc>) {
    let timestamp = dt.timestamp() as f64 + (dt.timestamp_subsec_micros() as f64 / 1_000_000.0);
    println!("{:<8} {:>20.6} s = {}", label, timestamp, dt.with_timezone(&Local));
    println!("         {:>20.6} s = {}", timestamp, dt.to_rfc3339());
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
        label,
        total_secs,
        sign,
        d,
        h,
        m,
        s,
        micros
    );
}
