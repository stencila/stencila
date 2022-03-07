use std::collections::HashSet;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use chrono_tz::{Tz, UTC};
use cron::Schedule;
use eyre::{bail, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while, take_while_m_n},
    character::{
        complete::{multispace0, multispace1},
        is_digit,
    },
    combinator::{fail, map, opt},
    multi::separated_list1,
    sequence::{delimited, preceded, tuple},
    IResult,
};

mod tz_abbreviations;

/// A cron schedule
///
/// See https://en.wikipedia.org/wiki/Cron for syntax (including extensions).
/// This has additional `seconds` fields for compatibility with `cron` crate.
#[derive(Debug, PartialEq, Eq)]
struct Cron {
    seconds: String,
    minutes: String,
    hours: String,
    days_of_month: String,
    months: String,
    days_of_week: String,
    years: String,
}

impl Default for Cron {
    fn default() -> Self {
        Self {
            seconds: "*".to_string(),
            minutes: "*".to_string(),
            hours: "*".to_string(),
            days_of_month: "*".to_string(),
            months: "*".to_string(),
            days_of_week: "*".to_string(),
            years: "*".to_string(),
        }
    }
}

impl Cron {
    fn from_cron(cron: &str) -> Self {
        let parts: Vec<String> = cron
            .split_whitespace()
            .map(|part| part.to_string())
            .collect();
        let star = "*".to_string();
        let get_or_star = |index| parts.get(index).unwrap_or(&star).clone();
        Self {
            seconds: get_or_star(0),
            minutes: get_or_star(1),
            hours: get_or_star(2),
            days_of_month: get_or_star(3),
            months: get_or_star(4),
            days_of_week: get_or_star(5),
            years: get_or_star(6),
        }
    }

    fn to_cron(&self) -> String {
        format!(
            "{} {} {} {} {} {} {}",
            self.seconds,
            self.minutes,
            self.hours,
            self.days_of_month,
            self.months,
            self.days_of_week,
            if self.years != "*" {
                self.years.as_str()
            } else {
                ""
            }
        )
        .trim()
        .to_string()
    }
}
/// Parse a string into a list of cron schedules in a specific timezone
pub fn parse(input: &str) -> Result<(Vec<Schedule>, Tz)> {
    let (crons, tz) = main(input)?;
    let mut schedules = Vec::new();
    for cron in crons {
        let cron = cron.to_cron();
        let schedule = match Schedule::from_str(&cron) {
            Ok(schedule) => schedule,
            Err(error) => bail!("Error parsing generated cron `{}`: {}", cron, error),
        };
        schedules.push(schedule);
    }
    Ok((schedules, tz))
}

/// Parse a list of cron phrases or expressions and optional timezone
fn main(input: &str) -> Result<(Vec<Cron>, Tz)> {
    match tuple((
        separated_list1(
            delimited(multispace1, tag("and"), multispace1),
            alt((cron, phrase)),
        ),
        opt(preceded(multispace1, timezone)),
    ))(input)
    {
        Ok((_, (crons, tz))) => Ok((crons, tz.unwrap_or(UTC))),
        Err(error) => bail!("Unable to parse input as schedule: {}", error),
    }
}

/// Parse a phrase describing a cron schedule
fn phrase(input: &str) -> IResult<&str, Cron> {
    map(
        separated_list1(multispace1, alt((every, at, time, on, dow_range, dow_list))),
        |crons| {
            let mut merged = Cron::default();
            for cron in crons {
                if cron.seconds != "*" {
                    merged.seconds = cron.seconds;
                }
                if cron.minutes != "*" {
                    merged.minutes = cron.minutes;
                }
                if cron.hours != "*" {
                    merged.hours = cron.hours;
                }
                if cron.days_of_month != "*" {
                    merged.days_of_month = cron.days_of_month;
                }
                if cron.months != "*" {
                    merged.months = cron.months;
                }
                if cron.days_of_week != "*" {
                    merged.days_of_week = cron.days_of_week;
                }
            }

            if merged.days_of_week != "*" && merged.hours == "*" {
                merged.hours = "0".to_string();
            }
            if merged.hours != "*" && merged.minutes == "*" {
                merged.minutes = "0".to_string();
            }
            if merged.minutes != "*" && merged.seconds == "*" {
                merged.seconds = "0".to_string();
            }

            merged
        },
    )(input)
}

/// Parse an "every" statement
fn every(input: &str) -> IResult<&str, Cron> {
    let minute = map(alt((tag_no_case("minute"), tag_no_case("min"))), |_| Cron {
        seconds: "0".to_string(),
        ..Default::default()
    });

    let hour = map(alt((tag_no_case("hour"), tag_no_case("hr"))), |_| Cron {
        seconds: "0".to_string(),
        minutes: "0".to_string(),
        ..Default::default()
    });

    let day = map(tag_no_case("day"), |_| Cron {
        seconds: "0".to_string(),
        minutes: "0".to_string(),
        hours: "0".to_string(),
        ..Default::default()
    });

    let week = map(tag_no_case("week"), |_| Cron {
        seconds: "0".to_string(),
        minutes: "0".to_string(),
        hours: "0".to_string(),
        days_of_week: "sun".to_string(),
        ..Default::default()
    });

    let duration = map(
        tuple((
            take_while(|c: char| is_digit(c as u8)),
            multispace0,
            alt((
                tag_no_case("seconds"),
                tag_no_case("secs"),
                tag_no_case("sec"),
                tag_no_case("s"),
                tag_no_case("minutes"),
                tag_no_case("mins"),
                tag_no_case("min"),
                tag_no_case("m"),
                tag_no_case("hours"),
                tag_no_case("hrs"),
                tag_no_case("hr"),
                tag_no_case("h"),
                tag_no_case("days"),
                tag_no_case("day"),
                tag_no_case("d"),
            )),
        )),
        |(num, _, unit): (&str, &str, &str)| -> Cron {
            if unit.starts_with('s') {
                Cron {
                    seconds: ["*/", num].concat(),
                    ..Default::default()
                }
            } else if unit.starts_with('m') {
                Cron {
                    seconds: "0".to_string(),
                    minutes: ["*/", num].concat(),
                    ..Default::default()
                }
            } else if unit.starts_with('h') {
                Cron {
                    seconds: "0".to_string(),
                    minutes: "0".to_string(),
                    hours: ["*/", num].concat(),
                    ..Default::default()
                }
            } else if unit.starts_with('d') {
                Cron {
                    seconds: "0".to_string(),
                    minutes: "0".to_string(),
                    hours: "0".to_string(),
                    days_of_month: ["*/", num].concat(),
                    ..Default::default()
                }
            } else {
                unreachable!()
            }
        },
    );

    preceded(
        tuple((tag_no_case("every"), multispace1)),
        alt((minute, hour, day, week, duration)),
    )(input)
}

/// Parse an "at" time-of-day statement
fn at(input: &str) -> IResult<&str, Cron> {
    preceded(tuple((tag_no_case("at"), multispace1)), time)(input)
}

/// Parse a time-of-day
fn time(input: &str) -> IResult<&str, Cron> {
    let hour = take_while_m_n(1, 2, |c: char| is_digit(c as u8));
    let minute = take_while_m_n(2, 2, |c: char| is_digit(c as u8));
    let second = take_while_m_n(2, 2, |c: char| is_digit(c as u8));
    map(
        tuple((
            hour,
            tag(":"),
            minute,
            opt(preceded(tag(":"), second)),
            opt(alt((tag_no_case("am"), tag_no_case("pm"),))),
        )),
        |(hour, _sep, minute, second, am_pm): (&str, &str, &str, Option<&str>, Option<&str>)| -> Cron {
            let hours = match am_pm {
                Some(value) => {
                    if value.to_lowercase() == "pm" {
                        let hour:u8 = hour.parse().unwrap_or(12);
                        let hour = if hour < 12 {
                            hour + 12
                        } else {
                            hour
                        };
                        format!("{}", hour)
                    } else {
                        hour.to_string()
                    }
                },
                _ => hour.to_string()
            };
            Cron {
                hours,
                minutes: minute.to_string(),
                seconds: second.map_or_else(|| "00".to_string(), String::from),
                ..Default::default()
            }
        },
    )(input)
}

/// Parse a "on" day-of-week statement
fn on(input: &str) -> IResult<&str, Cron> {
    preceded(
        tuple((tag_no_case("on"), multispace1)),
        alt((dow_range, dow_list)),
    )(input)
}

/// Parse a day-of-week list
fn dow_list(input: &str) -> IResult<&str, Cron> {
    map(
        separated_list1(delimited(multispace0, tag(","), multispace0), dow),
        |crons| {
            // Collect into hash set first to ensure uniqueness
            let days: HashSet<String> = crons.into_iter().map(|cron| cron.days_of_week).collect();
            let mut days: Vec<String> = days.into_iter().collect();
            days.sort_by_key(|day| dow_order(day));
            Cron {
                days_of_week: days.join(","),
                ..Default::default()
            }
        },
    )(input)
}

/// Parse a day-of-week range
fn dow_range(input: &str) -> IResult<&str, Cron> {
    map(
        preceded(
            opt(tuple((tag_no_case("from"), multispace1))),
            tuple((
                dow,
                alt((
                    delimited(multispace0, tag("-"), multispace0),
                    delimited(
                        multispace1,
                        alt((tag_no_case("to"), tag_no_case("through"))),
                        multispace1,
                    ),
                )),
                dow,
            )),
        ),
        |(begin, _delim, end)| {
            let begin = begin.days_of_week;
            let end = end.days_of_week;
            Cron {
                days_of_week: [&begin, "-", &end].concat(),
                ..Default::default()
            }
        },
    )(input)
}

/// Get the cron order of the day of the week
fn dow_order(day: &str) -> u8 {
    match day {
        "sun" => 0,
        "mon" => 1,
        "tue" => 2,
        "wed" => 3,
        "thu" => 4,
        "fri" => 5,
        "sat" => 6,
        _ => 7,
    }
}

/// Parse a day of week
fn dow(input: &str) -> IResult<&str, Cron> {
    let sun = map(alt((tag_no_case("sunday"), tag_no_case("sun"))), |_| "sun");
    let mon = map(alt((tag_no_case("monday"), tag_no_case("mon"))), |_| "mon");
    let tue = map(
        alt((
            tag_no_case("tuesday"),
            tag_no_case("tues"),
            tag_no_case("tue"),
        )),
        |_| "tue",
    );
    let wed = map(
        alt((
            tag_no_case("wednesday"),
            tag_no_case("wednes"),
            tag_no_case("wed"),
        )),
        |_| "wed",
    );
    let thu = map(
        alt((
            tag_no_case("thursday"),
            tag_no_case("thurs"),
            tag_no_case("thur"),
            tag_no_case("thu"),
        )),
        |_| "thu",
    );
    let fri = map(alt((tag_no_case("friday"), tag_no_case("fri"))), |_| "fri");
    let sat = map(alt((tag_no_case("saturday"), tag_no_case("sat"))), |_| {
        "sat"
    });

    map(
        alt((sun, mon, tue, wed, thu, fri, sat)),
        |day: &str| -> Cron {
            Cron {
                days_of_week: day.to_owned(),
                ..Default::default()
            }
        },
    )(input)
}

/// Parse a cron expression
fn cron(input: &str) -> IResult<&str, Cron> {
    match Schedule::from_str(input) {
        Ok(schedule) => {
            let cron = schedule.to_string();
            let cron = Cron::from_cron(&cron);
            Ok((input, cron))
        }
        Err(..) => fail(input),
    }
}

/// Parse a timezone
fn timezone(input: &str) -> IResult<&str, Tz> {
    let name = match tz_abbreviations::LIST.get(input) {
        Some(name) => name,
        None => input,
    };

    if let Ok(tz) = Tz::from_str(name) {
        return Ok((input, tz));
    }

    fail(input)
}

/// Get the next time (in UTC) from amongst a list of schedules in a specific timezone
pub fn next(schedules: &[Schedule], tz: &Tz) -> Option<DateTime<Utc>> {
    let mut times = schedules
        .iter()
        .filter_map(|schedule| schedule.upcoming(*tz).next())
        .collect::<Vec<_>>();
    times.sort();
    times.first().map(|time| time.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use chrono_tz::{
        Europe::Madrid,
        Pacific::Auckland,
        US::{Central, Eastern},
    };

    use super::*;

    #[test]
    fn time() -> Result<()> {
        assert_eq!(parse("1:23:45")?.0[0], Schedule::from_str("45 23 1 * * *")?);
        assert_eq!(
            parse("1:23:45pm")?.0[0],
            Schedule::from_str("45 23 13 * * *")?
        );
        assert_eq!(parse("1:23am")?.0[0], Schedule::from_str("00 23 1 * * *")?);
        assert_eq!(parse("1:23pm")?.0[0], Schedule::from_str("00 23 13 * * *")?);
        assert_eq!(
            parse("13:23pm")?.0[0],
            Schedule::from_str("00 23 13 * * *")?
        );
        assert_eq!(parse("1:23PM")?.0[0], Schedule::from_str("00 23 13 * * *")?);
        Ok(())
    }

    #[test]
    fn at() -> Result<()> {
        assert_eq!(
            parse("at 00:00:00")?.0[0],
            Schedule::from_str("00 00 00 * * *")?
        );
        assert_eq!(
            parse("at 01:02")?.0[0],
            Schedule::from_str("00 02 01 * * *")?
        );

        Ok(())
    }

    #[test]
    fn every() -> Result<()> {
        assert_eq!(parse("every min")?.0[0], Schedule::from_str("0 * * * * *")?);
        assert_eq!(
            parse("every hour")?.0[0],
            Schedule::from_str("0 0 * * * *")?
        );
        assert_eq!(parse("every day")?.0[0], Schedule::from_str("0 0 0 * * *")?);
        assert_eq!(
            parse("every week")?.0[0],
            Schedule::from_str("0 0 0 * * sun")?
        );

        assert_eq!(
            parse("every 10 secs")?.0[0],
            Schedule::from_str("*/10 * * * * *")?
        );
        assert_eq!(
            parse("every 30 mins")?.0[0],
            Schedule::from_str("0 */30 * * * *")?
        );
        assert_eq!(
            parse("every 2 hrs")?.0[0],
            Schedule::from_str("0 0 */2 * * *")?
        );
        assert_eq!(
            parse("every 2 days")?.0[0],
            Schedule::from_str("0 0 0 */2 * *")?
        );

        Ok(())
    }

    #[test]
    fn day_of_week() -> Result<()> {
        for day in [
            "sun",
            "sunday",
            "Sun",
            "Sunday",
            "mon",
            "monday",
            "tue",
            "tues",
            "tuesday",
            "wed",
            "wednesday",
            "thu",
            "thur",
            "thurs",
            "thursday",
            "fri",
            "friday",
            "sat",
            "saturday",
        ] {
            assert_eq!(
                parse(day)?.0[0],
                Schedule::from_str(&["0 0 0 * * ", &day[0..3].to_lowercase()].concat())?
            );
        }
        Ok(())
    }

    #[test]
    fn day_of_week_list() -> Result<()> {
        let target = Schedule::from_str("0 0 0 * * sun,mon")?;
        for exp in [
            "sun,mon",
            "sun, mon",
            "sunday, mon",
            "mon, sun",
            "sun, monday",
            "mon,sun,mon, sunday,monday",
        ] {
            assert_eq!(parse(exp)?.0[0], target);
        }

        let target = Schedule::from_str("0 0 0 * * tue,thu,fri")?;
        for exp in ["tue,thu,fri", "friday, thu, tuesday"] {
            assert_eq!(parse(exp)?.0[0], target);
        }

        Ok(())
    }

    #[test]
    fn day_of_week_range() -> Result<()> {
        let target = Schedule::from_str("0 0 0 * * mon-fri")?;
        for exp in [
            "mon-fri",
            "mon - fri",
            "mon to fri",
            "monday through friday",
            "from mon to fri",
            "from     monday -   friday",
        ] {
            assert_eq!(parse(exp)?.0[0], target);
        }

        Ok(())
    }

    #[test]
    fn phrase() -> Result<()> {
        let target = Schedule::from_str("00 23 1 * * mon")?;
        for exp in [
            "1:23 mon",
            "at 1:23 mon",
            "at 1:23 on monday",
            "on monday at 1:23",
            "mon at 1:23",
            "mon 1:23:00",
        ] {
            assert_eq!(parse(exp)?.0[0], target);
        }

        Ok(())
    }

    #[test]
    fn cron() -> Result<()> {
        for exp in [
            "* * * * * *",
            "* 0 * * * *",
            "* * * * * sun",
            "* * * * * * 2030",                             // with optional year
            "0 30 9,12,15 1,15 May-Aug Mon,Wed,Fri 2018/2", // from `cron` crate README
        ] {
            assert_eq!(parse(exp)?.0[0], Schedule::from_str(exp)?);
        }
        Ok(())
    }

    #[test]
    fn timezones() -> Result<()> {
        assert_eq!(timezone("UTC")?.1, UTC);
        assert_eq!(timezone("Europe/Madrid")?.1, Madrid);
        assert_eq!(timezone("Pacific/Auckland")?.1, Auckland);
        assert_eq!(timezone("ET")?.1, Eastern);
        Ok(())
    }

    #[test]
    fn multiple() -> Result<()> {
        let target = vec![
            Schedule::from_str("00 34 12 * * tue")?,
            Schedule::from_str("01 12 09 * * fri")?,
        ];

        let (schedules, tz) = parse("tue at 12:34 and at 09:12:01 on fri")?;
        assert_eq!(schedules, target);
        assert_eq!(tz, UTC);

        let (schedules, tz) = parse("tue at 12:34 and fri at 09:12:01 CT")?;
        assert_eq!(schedules, target);
        assert_eq!(tz, Central);

        Ok(())
    }

    #[test]
    fn iterate() -> Result<()> {
        let (schedules, timezone) = parse("every minute and every day")?;
        let time = next(&schedules, &timezone);
        //println!("{:?}", time);
        assert!(time.is_some());

        Ok(())
    }
}
