/// Escape LaTeX-special characters **except** inside real math.
///
/// Heuristics
/// ----------
/// * `$` or `$$` start math **only if** a matching delimiter exists
///   later in the string; otherwise they’re treated as literal dollars
///   and escaped (`\$`).
/// * A single `$` followed by a digit, space, or one of `#%&_~$` is
///   considered literal (covers prices like `$5`, the troublesome
///   `_$#1`, etc.).
/// * When we do enter math we remember whether we opened with `$` or
///   `$$`, so we don’t exit until we see the *same* delimiter.
pub fn escape_latex(input: &str) -> String {
    // Mapping for ordinary text-mode escapes.
    const MAP: [(char, &str); 10] = [
        ('\\', r"\textbackslash{}"),
        ('{', r"\{"),
        ('}', r"\}"),
        ('#', r"\#"),
        ('$', r"\$"),
        ('%', r"\%"),
        ('&', r"\&"),
        ('~', r"\textasciitilde{}"),
        ('_', r"\_"),
        ('^', r"\textasciicircum{}"),
    ];

    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let bytes = input.as_bytes();
    let mut in_math = false;
    let mut math_delim = ""; // "$" or "$$"

    while i < bytes.len() {
        let c = bytes[i] as char;

        if !in_math && c == '$' {
            // ---- decide whether this starts math or is a literal $ ----
            let next = if i + 1 < bytes.len() {
                bytes[i + 1] as char
            } else {
                '\0'
            };
            if next == '$' {
                // Candidate `$$`
                let rest = &input[i + 2..];
                if rest.contains("$$") {
                    in_math = true;
                    math_delim = "$$";
                    out.push_str("$$");
                    i += 2;
                    continue;
                }
            } else {
                // Candidate single `$`
                let rest = &input[i + 1..];
                if !next.is_ascii_digit()
                    && !next.is_ascii_whitespace()
                    && !"#%&_~$".contains(next)
                    && rest.contains('$')
                {
                    in_math = true;
                    math_delim = "$";
                    out.push('$');
                    i += 1;
                    continue;
                }
            }
            // Literal dollar(s)
            out.push_str(r"\$");
            if next == '$' {
                out.push_str(r"\$");
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        if in_math {
            // ---- inside math ------------------------------------------------
            if c == '$' {
                if math_delim == "$$" && i + 1 < bytes.len() && bytes[i + 1] == b'$' {
                    out.push_str("$$");
                    in_math = false;
                    i += 2;
                    continue;
                } else if math_delim == "$" {
                    out.push('$');
                    in_math = false;
                    i += 1;
                    continue;
                }
            }
            out.push(c);
            i += 1;
            continue;
        }

        // ---- text mode: ordinary escapes -----------------------------------
        if let Some((_, repl)) = MAP.iter().find(|(ch, _)| *ch == c) {
            out.push_str(repl);
        } else {
            out.push(c);
        }
        i += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::escape_latex;

    #[test]
    fn leaves_math_alone() {
        let raw = "Temp is $^{\\circ}$C & costs $5";
        let want = r"Temp is $^{\circ}$C \& costs \$5";
        assert_eq!(escape_latex(raw), want);
    }

    #[test]
    fn escapes_outside_math_only() {
        let raw = r"\price_$#1 ~ ok $$x^2$$";
        let want = r"\textbackslash{}price\_\$\#1 \textasciitilde{} ok $$x^2$$";
        assert_eq!(escape_latex(raw), want);
    }
}
