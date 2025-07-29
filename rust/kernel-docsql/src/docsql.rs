use kernel_jinja::kernel::common::{
    once_cell::sync::Lazy,
    regex::{Captures, Regex},
};

/// Encode DocsQL filter arguments into valid MiniJinja keyword arguments
///
/// Uses single digit codes and spacing to ensure that the code stays the same length.
pub(super) fn encode_filters(code: &str) -> String {
    static FILTERS: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"((?:\(|,)\s*)(\*|(?:\.[a-zA-Z][\w_]*))\s*(==|\!=|<=|<|>=|>|\~=|\~\!|\^=|\$=|in|has|=)\s*")
            .expect("invalid regex")
    });

    let code = FILTERS.replace_all(code, |captures: &Captures| {
        let before = &captures[1];
        let var = &captures[2];
        let op = match &captures[3] {
            "=" | "==" => "",
            "!=" => "0",
            "<" => "1",
            "<=" => "2",
            ">" => "3",
            ">=" => "4",
            "~=" => "5",
            "~!" => "6",
            "^=" => "7",
            "$=" => "8",
            "in" => "9",
            "has" => "_",
            echo => echo,
        };

        let var = match var {
            "*" => "_C", // Count
            _ => var.trim_start_matches("."),
        };

        let spaces = captures[0]
            .len()
            .saturating_sub(before.len() + var.len() + op.len() + 1);
        let spaces = " ".repeat(spaces);

        [before, var, op, &spaces, "="].concat()
    });

    static SUBQUERY: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"((?:\(|,)\s*)\.\.\.([a-zA-Z][\w_]*)").expect("invalid regex"));

    SUBQUERY
        .replace_all(&code, |captures: &Captures| {
            let pre = &captures[1];
            let func = &captures[2];
            [pre, "_=_", func].concat()
        })
        .into()
}

/// Decode a Minijinja argument name into a property name and operator
/// 
/// The inverse of `encode_filter` for a single argument.
pub(super) fn decode_filter(arg_name: &str) -> (&str, &str) {
    if arg_name.len() > 1 {
        if let Some(last_char) = arg_name.chars().last() {
            let trimmed = &arg_name[..arg_name.len() - 1];
            match last_char {
                '0' => (trimmed, "!="),
                '1' => (trimmed, "<"),
                '2' => (trimmed, "<="),
                '3' => (trimmed, ">"),
                '4' => (trimmed, ">="),
                '5' => (trimmed, "~="),
                '6' => (trimmed, "~!"),
                '7' => (trimmed, "^="),
                '8' => (trimmed, "$="),
                '9' => (trimmed, "in"),
                '_' => (trimmed, "has"),
                _ => (arg_name, "=="),
            }
        } else {
            (arg_name, "==")
        }
    } else {
        (arg_name, "==")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn transform_filters() {
        use super::encode_filters as t;

        assert_eq!(t(""), "");
        assert_eq!(t(".a"), ".a");

        assert_eq!(t("(.a = 1)"), "(a   =1)");
        assert_eq!(t("(.a= 1)"), "(a  =1)");
        assert_eq!(t("(.a =1)"), "(a  =1)");
        assert_eq!(t("(.a=1)"), "(a =1)");

        assert_eq!(t("(.a == 1)"), "(a    =1)");
        assert_eq!(t("(.a== 1)"), "(a   =1)");
        assert_eq!(t("(.a ==1)"), "(a   =1)");
        assert_eq!(t("(.a==1)"), "(a  =1)");

        assert_eq!(t("(.a < 1)"), "(a1  =1)");
        assert_eq!(t("(.a< 1)"), "(a1 =1)");
        assert_eq!(t("(.a <1)"), "(a1 =1)");
        assert_eq!(t("(.a<1)"), "(a1=1)");

        assert_eq!(t("(.abc ~! 'regex')"), "(abc6   ='regex')");
        assert_eq!(t("(.abc~! 'regex')"), "(abc6  ='regex')");
        assert_eq!(t("(.abc ~!'regex')"), "(abc6  ='regex')");
        assert_eq!(t("(.abc~!'regex')"), "(abc6 ='regex')");

        assert_eq!(t("(.a != 1)"), "(a0   =1)");
        assert_eq!(t("(.a < 1)"), "(a1  =1)");
        assert_eq!(t("(.a <= 1)"), "(a2   =1)");
        assert_eq!(t("(.a > 1)"), "(a3  =1)");
        assert_eq!(t("(.a >= 1)"), "(a4   =1)");
        assert_eq!(t("(.a ~= 1)"), "(a5   =1)");
        assert_eq!(t("(.a ~! 1)"), "(a6   =1)");
        assert_eq!(t("(.a ^= 1)"), "(a7   =1)");
        assert_eq!(t("(.a $= 1)"), "(a8   =1)");
        assert_eq!(t("(.a in 1)"), "(a9   =1)");
        assert_eq!(t("(.a has 1)"), "(a_    =1)");

        assert_eq!(
            t("(.a != 1, .b < 1,.c has 1)"),
            "(a0   =1, b1  =1,c_    =1)"
        );

        assert_eq!(t("(above)"), "(above)");
        assert_eq!(t("(below, .a != 1)"), "(below, a0   =1)");

        assert_eq!(t("(* == 1)"), "(_C  =1)");
        assert_eq!(t("(* <  1)"), "(_C1 =1)");
        assert_eq!(t("(* > 1)"), "(_C3=1)");
        assert_eq!(t("(*>=1)"), "(_C4=1)");
    }
}
