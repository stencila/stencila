use std::collections::HashSet;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use chrono_tz::{Tz, TZ_VARIANTS, UTC};
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
use tz_abbreviations::TZ_ABBREVIATIONS;

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
        separated_list1(
            multispace1,
            alt((
                every, at, time, hour_range, hour_list, on, dow_range, dow_list,
            )),
        ),
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

            if merged.days_of_week != "*"
                && merged.hours == "*"
                && merged.minutes == "*"
                && merged.seconds == "*"
            {
                merged.hours = "0".into();
                merged.minutes = "0".into();
                merged.seconds = "0".into();
            }

            if merged.hours != "*" && merged.minutes == "*" && merged.seconds == "*" {
                merged.minutes = "0".into();
                merged.seconds = "0".into();
            }

            if merged.minutes != "*" && merged.seconds == "*" {
                merged.seconds = "0".into();
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
    preceded(
        tuple((tag_no_case("at"), multispace1)),
        alt((time, hour_list)),
    )(input)
}

/// Parse an "am/pm" string
fn am_pm(input: &str) -> IResult<&str, String> {
    map(
        alt((tag_no_case("am"), tag_no_case("pm"))),
        |am_pm: &str| am_pm.to_lowercase(),
    )(input)
}

/// Apply an "am/pm" modifier to hours string
fn am_pm_apply(hour: &str, am_pm: Option<String>) -> String {
    let hour = match am_pm {
        Some(am_pm) => {
            if am_pm == "pm" {
                let hour: u8 = hour.parse().unwrap_or(12);
                let hour = if hour < 12 { hour + 12 } else { hour };
                format!("{}", hour)
            } else {
                hour.to_string()
            }
        }
        _ => hour.to_string(),
    };
    if hour == "00" {
        "0".to_string()
    } else {
        hour
    }
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
            opt(am_pm),
        )),
        |(hour, _sep, minute, second, am_pm): (&str, &str, &str, Option<&str>, Option<String>)| -> Cron {
            Cron {
                hours: am_pm_apply(hour, am_pm),
                minutes: (if minute == "00" {"0"} else {minute}).to_string(),
                seconds: second.map_or_else(|| "0", |second| if second == "00" {"0"} else {second}).to_string(),
                ..Default::default()
            }
        },
    )(input)
}

/// Parse an hour-of-day
fn hour(input: &str) -> IResult<&str, Cron> {
    map(
        tuple((
            take_while_m_n(1, 2, |c: char| is_digit(c as u8)),
            opt(am_pm),
        )),
        |(hour, am_pm)| Cron {
            hours: am_pm_apply(hour, am_pm),
            ..Default::default()
        },
    )(input)
}

/// Parse a hour-of-day list
fn hour_list(input: &str) -> IResult<&str, Cron> {
    map(
        separated_list1(delimited(multispace0, tag(","), multispace0), hour),
        |crons| {
            // Collect into hash set to ensure uniqueness
            let hours: HashSet<String> = crons.into_iter().map(|cron| cron.hours).collect();
            let mut hours: Vec<String> = hours.into_iter().collect();
            hours.sort_by_key(|hour| hour.parse::<u8>().unwrap_or_default());
            Cron {
                hours: hours.join(","),
                ..Default::default()
            }
        },
    )(input)
}

/// Parse a hour-of-day range
fn hour_range(input: &str) -> IResult<&str, Cron> {
    map(
        alt((
            preceded(
                opt(tuple((tag_no_case("from"), multispace1))),
                tuple((
                    hour,
                    alt((
                        delimited(multispace0, tag("-"), multispace0),
                        delimited(
                            multispace1,
                            alt((tag_no_case("to"), tag_no_case("through"))),
                            multispace1,
                        ),
                    )),
                    hour,
                )),
            ),
            preceded(
                opt(tuple((tag_no_case("between"), multispace1))),
                tuple((
                    hour,
                    delimited(multispace0, tag_no_case("and"), multispace0),
                    hour,
                )),
            ),
        )),
        |(begin, _delim, end)| {
            let begin = begin.hours;
            let end = end.hours;
            Cron {
                hours: [&begin, "-", &end].concat(),
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
///
/// Ignores any trailing "time" e.g. "Auckland time" and then search using
/// abbreviations, the full, two-part, name, or the second (city) part, of the timezone
fn timezone(input: &str) -> IResult<&str, Tz> {
    let name = match input.trim().strip_suffix("time") {
        Some(name) => name,
        None => input,
    }
    .trim();

    let name = match TZ_ABBREVIATIONS.get(name.to_uppercase().as_str()) {
        Some(name) => name,
        None => name,
    };

    if let Ok(tz) = Tz::from_str(name) {
        return Ok((input, tz));
    }

    let name = name.to_lowercase();
    for tz in TZ_VARIANTS {
        let city = tz
            .to_string()
            .split('/')
            .last()
            .unwrap_or_default()
            .to_lowercase();
        if city == name {
            return Ok((input, tz));
        }
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
        Asia::Kathmandu,
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
        assert_eq!(parse("1:23am")?.0[0], Schedule::from_str("0 23 1 * * *")?);
        assert_eq!(parse("1:23pm")?.0[0], Schedule::from_str("0 23 13 * * *")?);
        assert_eq!(parse("13:23pm")?.0[0], Schedule::from_str("0 23 13 * * *")?);
        assert_eq!(parse("1:23PM")?.0[0], Schedule::from_str("0 23 13 * * *")?);
        Ok(())
    }

    #[test]
    fn hour() -> Result<()> {
        assert_eq!(parse("1")?.0[0], Schedule::from_str("0 0 1 * * *")?);
        assert_eq!(parse("1am")?.0[0], Schedule::from_str("0 0 1 * * *")?);
        assert_eq!(parse("1pm")?.0[0], Schedule::from_str("0 0 13 * * *")?);
        assert_eq!(parse("22pm")?.0[0], Schedule::from_str("0 0 22 * * *")?);

        Ok(())
    }

    #[test]
    fn hour_list() -> Result<()> {
        assert_eq!(parse("1,2,3")?.0[0], Schedule::from_str("0 0 1,2,3 * * *")?);
        assert_eq!(
            parse("1am, 1pm, 20")?.0[0],
            Schedule::from_str("0 0 1,13,20 * * *")?
        );

        Ok(())
    }

    #[test]
    fn hour_range() -> Result<()> {
        let target = Schedule::from_str("0 0 2-14 * * *")?;
        for exp in [
            "2-14",
            "2am-14",
            "2am - 2pm",
            "2 to 14",
            "from 2am to 2pm",
            "between 2 and 14",
        ] {
            assert_eq!(parse(exp)?.0[0], target);
        }

        Ok(())
    }

    #[test]
    fn at() -> Result<()> {
        assert_eq!(
            parse("at 00:00:00")?.0[0],
            Schedule::from_str("0 0 0 * * *")?
        );
        assert_eq!(
            parse("at 01:02")?.0[0],
            Schedule::from_str("0 02 01 * * *")?
        );
        assert_eq!(parse("at 1am")?.0[0], Schedule::from_str("0 0 1 * * *")?);
        assert_eq!(
            parse("at 1, 2, 10")?.0[0],
            Schedule::from_str("0 0 1,2,10 * * *")?
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

        assert_eq!(
            parse("every 2 hours monday to friday")?.0[0],
            Schedule::from_str("0 0 */2 * * mon-fri")?
        );
        assert_eq!(
            parse("every 30 mins tue,wed")?.0[0],
            Schedule::from_str("0 */30 * * * tue,wed")?
        );
        assert_eq!(
            parse("every 5s from sun to thur")?.0[0],
            Schedule::from_str("*/5 * * * * sun-thu")?
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
        let target = Schedule::from_str("0 23 1 * * mon")?;
        for exp in [
            "1:23 mon",
            "mon 1:23",
            "at 1:23 mon",
            "mon at 1:23",
            "at 1:23 on monday",
            "on monday at 1:23",
            "monday at 1:23am",
        ] {
            assert_eq!(parse(exp)?.0[0], target);
        }

        let target = Schedule::from_str("0 */5 9-17 * * mon-fri")?;
        for exp in [
            "Every 5 minutes between 9AM and 5PM Monday to Friday",
            "Between 9AM and 5PM every 5mins Mon-Fri",
            "every 5mins 9-17 mon-fri",
            "mon-fri 9-17 every 5mins",
            "0 */5 9-17 * * mon-fri",
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

        assert_eq!(timezone("Madrid")?.1, Madrid);
        assert_eq!(timezone("Europe/Madrid")?.1, Madrid);
        assert_eq!(timezone("  Europe/Madrid    time ")?.1, Madrid);

        assert_eq!(timezone("npt")?.1, Kathmandu);
        assert_eq!(timezone("npt time")?.1, Kathmandu);
        assert_eq!(timezone("Kathmandu")?.1, Kathmandu);
        assert_eq!(timezone("Asia/Kathmandu")?.1, Kathmandu);

        assert_eq!(timezone("auckland")?.1, Auckland);
        assert_eq!(timezone("Auckland")?.1, Auckland);
        assert_eq!(timezone("Auckland time")?.1, Auckland);
        assert_eq!(timezone("Pacific/Auckland")?.1, Auckland);

        assert_eq!(timezone("ET")?.1, Eastern);
        assert_eq!(timezone("US/Eastern")?.1, Eastern);

        Ok(())
    }

    #[test]
    fn multiple() -> Result<()> {
        let target = vec![
            Schedule::from_str("0 0 13 * * tue")?,
            Schedule::from_str("01 12 09 * * fri")?,
        ];

        let (schedules, tz) = parse("on tue at 1pm and at 09:12:01 on fri")?;
        assert_eq!(schedules, target);
        assert_eq!(tz, UTC);

        let (schedules, tz) = parse("tue at 13 and fri at 09:12:01 ct")?;
        assert_eq!(schedules, target);
        assert_eq!(tz, Central);

        let (schedules, tz) = parse("13:00 tue and 09:12:01 fri NPT")?;
        assert_eq!(schedules, target);
        assert_eq!(tz, Kathmandu);

        let (schedules, tz) = parse("At 1pm Tuesday and on Friday at 09:12:01 Kathmandu")?;
        assert_eq!(schedules, target);
        assert_eq!(tz, Kathmandu);

        let (schedules, tz) = parse("1pm Tuesday and 09:12:01 Friday Auckland time")?;
        assert_eq!(schedules, target);
        assert_eq!(tz, Auckland);

        let (schedules, tz) = parse("tue at 13:00 and fri at 09:12:01 auckland time")?;
        assert_eq!(schedules, target);
        assert_eq!(tz, Auckland);

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
