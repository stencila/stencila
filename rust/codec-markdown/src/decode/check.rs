use codec::{common::strum::Display, format::Format, Message, MessageLevel, Messages};

/// Check Markdown prior to decoding
pub fn check(md: &str, _format: &Format) -> Messages {
    let mut messages = Messages::default();

    // Iterate over lines finding those that should have pairs
    // or which use unrecognized directive names
    let mut colon_fences = Vec::new();
    let mut backtick_fences = Vec::new();
    let mut dollar_fences = Vec::new();
    for (index, line) in md.lines().enumerate() {
        // Count the number of successive leading colons, backticks, or dollars
        let mut colons = 0;
        let mut backticks = 0;
        let mut dollars = 0;
        let mut trailing_chars = false;
        for char in line.chars() {
            if char == ':' {
                if backticks == 0 && dollars == 0 {
                    colons += 1;
                } else {
                    break;
                }
            } else if char == '`' {
                if colons == 0 && dollars == 0 {
                    backticks += 1;
                } else {
                    break;
                }
            } else if char == '$' {
                if colons == 0 && backticks == 0 {
                    dollars += 1;
                } else {
                    break;
                }
            } else if char != ' ' && char != '\t' {
                if colons >= 3 || backticks >= 3 {
                    trailing_chars = true;
                }
                break;
            }
        }

        // Determine if an opening or closing line
        #[derive(Display)]
        enum Fence {
            No,
            #[strum(to_string = "opening colon fence")]
            OpeningColons(u32),
            #[strum(to_string = "closing colon fence")]
            ClosingColons(u32),
            #[strum(to_string = "separating colon fence")]
            SeparatingColons(u32),
            #[strum(to_string = "opening backtick fence")]
            OpeningBackticks(u32),
            #[strum(to_string = "closing backtick fence")]
            ClosingBackticks(u32),
            #[strum(to_string = "dollar fence")]
            Dollars(u32),
        }
        let fence = if colons >= 3 {
            // Is this a self-closing, or separating, colon fence?
            let (self_closing, separating) = if trailing_chars {
                let line = line.trim_start_matches(':').trim();
                if line.starts_with("include")
                    || line.starts_with("call")
                    || line.starts_with("chat")
                    || line.starts_with("prompt")
                    || line.ends_with(":::")
                    || line.ends_with(">>>")
                {
                    (true, false)
                } else if line.starts_with("elif") || line.starts_with("else") {
                    (false, true)
                } else {
                    (false, false)
                }
            } else {
                (false, false)
            };

            if self_closing {
                Fence::No
            } else if separating {
                Fence::SeparatingColons(colons)
            } else if trailing_chars {
                Fence::OpeningColons(colons)
            } else {
                Fence::ClosingColons(colons)
            }
        } else if backticks >= 3 {
            if trailing_chars {
                Fence::OpeningBackticks(backticks)
            } else {
                Fence::ClosingBackticks(backticks)
            }
        } else if dollars >= 2 {
            Fence::Dollars(dollars)
        } else {
            Fence::No
        };

        match fence {
            Fence::OpeningColons(..) => colon_fences.push((index, fence)),
            Fence::SeparatingColons(separating_colons) => {
                match colon_fences.last() {
                    Some(&(opening_line, Fence::OpeningColons(opening_colons))) => {
                        if separating_colons != opening_colons {
                            messages.push(warning(index,
                                format!(
                                    "Number of separating colons differs from opening colons on line {opening_line} ({separating_colons} != {opening_colons})",
                                    opening_line = opening_line + 1
                            )));
                        }
                        colon_fences.pop();
                    }
                    Some(&(
                        separating_line,
                        Fence::SeparatingColons(previous_separating_colons),
                    )) => {
                        if separating_colons != previous_separating_colons {
                            messages.push(warning(index,
                                format!(
                                    "Number of separating colons differs from opening colons on line {separating_line} ({separating_colons} != {previous_separating_colons})",
                                    separating_line = separating_line + 1
                            )));
                        }
                        colon_fences.pop();
                    }
                    Some(..) => {}
                    None => messages.push(error(index, "Unpaired separating colon fence")),
                }
                colon_fences.push((index, fence));
            }
            Fence::ClosingColons(closing_colons) => match colon_fences.last() {
                Some(&(opening_line, Fence::OpeningColons(opening_colons))) => {
                    if closing_colons != opening_colons {
                        messages.push(warning(index,
                                format!(
                                    "Number of closing colons differs from opening colons on line {opening_line} ({closing_colons} != {opening_colons})",
                                    opening_line = opening_line + 1
                            )));
                    }
                    colon_fences.pop();
                }
                Some(&(separating_line, Fence::SeparatingColons(separating_colons))) => {
                    if closing_colons != separating_colons {
                        messages.push(warning(index,
                                format!(
                                    "Number of closing colons differs from separating colons on line {separating_line} ({closing_colons} != {separating_colons})",
                                    separating_line = separating_line + 1
                            )));
                    }
                    colon_fences.pop();
                }
                Some(..) => {}
                None => messages.push(error(index, "Unpaired closing colon fence")),
            },
            Fence::OpeningBackticks(..) => backtick_fences.push((index, fence)),
            Fence::ClosingBackticks(closing_backticks) => {
                match backtick_fences.last() {
                    Some(&(opening_line, Fence::OpeningBackticks(opening_backticks))) => {
                        if closing_backticks != opening_backticks {
                            messages.push(warning(index,
                                format!(
                                    "Number of closing backticks differs from opening backticks on line {opening_line} ({closing_backticks} != {opening_backticks})",
                                    opening_line = opening_line + 1
                            )));
                        }
                        // Pop off the last opening fence
                        backtick_fences.pop();
                    }
                    Some(..) => {}
                    None => {
                        // Note that plain backticks (with no trailing chars) may be opening.
                        // So in this case, where there is not a paired opening, push this onto
                        backtick_fences.push((index, Fence::OpeningBackticks(closing_backticks)));
                    }
                }
            }
            Fence::Dollars(dollars) => {
                match dollar_fences.last() {
                    Some(&(opening_line, Fence::Dollars(opening_dollars))) => {
                        if dollars != opening_dollars {
                            messages.push(warning(index,
                                format!(
                                    "Number of closing dollars differs from opening dollars on line {opening_line} ({dollars} != {opening_dollars})",
                                    opening_line = opening_line + 1
                            )));
                        }
                        // Pop off the last opening fence
                        dollar_fences.pop();
                    }
                    Some(..) => {}
                    None => dollar_fences.push((index, Fence::Dollars(dollars))),
                }
            }
            Fence::No => {}
        }
    }

    for (line, fence) in colon_fences {
        messages.push(error(line, format!("Unpaired {fence}")))
    }

    for (line, fence) in backtick_fences {
        messages.push(error(line, format!("Unpaired {fence}")))
    }

    for (line, ..) in dollar_fences {
        messages.push(error(line, "Unpaired opening dollar fence"))
    }

    messages
}

fn warning<S: AsRef<str>>(start_line: usize, message: S) -> Message {
    Message {
        level: MessageLevel::Warning,
        message: message.as_ref().into(),
        start_line: Some(start_line),
        ..Default::default()
    }
}

fn error<S: AsRef<str>>(start_line: usize, message: S) -> Message {
    Message {
        level: MessageLevel::Error,
        message: message.as_ref().into(),
        start_line: Some(start_line),
        ..Default::default()
    }
}
