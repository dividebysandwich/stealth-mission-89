//! A radio call as the voice should say it: runways, headings and callsign numbers digit by
//! digit ("two seven", "niner"), altitudes in thousands ("two thousand"), distances as numbers
//! ("twelve miles").

/// Words after which a number is a quantity, said as a number.
const COUNTED: &[&str] = &["mile", "miles", "knots", "feet", "minutes", "seconds"];

const DIGITS: [&str; 10] =
    ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "niner"];

pub fn spoken(text: &str) -> String {
    let text = text.replace("glidepath", "glide path");
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if !c.is_ascii_digit() {
            // A hyphen in a callsign ("1-1") or a name.
            out.push(if c == '-' { ' ' } else { c });
            i += 1;
            continue;
        }
        // A number, its thousands separated by commas.
        let start = i;
        while i < chars.len()
            && (chars[i].is_ascii_digit()
                || (chars[i] == ',' && chars.get(i + 1).is_some_and(char::is_ascii_digit)))
        {
            i += 1;
        }
        let token: String = chars[start..i].iter().collect();
        let digits: String = token.chars().filter(char::is_ascii_digit).collect();
        let next: String = chars[i..]
            .iter()
            .skip_while(|c| **c == ' ')
            .take_while(|c| c.is_alphabetic())
            .collect();
        let counted = token.contains(',') || COUNTED.contains(&next.as_str());
        match digits.parse::<u32>() {
            Ok(n) if counted && n < 100_000 => out.push_str(&number(n)),
            _ => {
                let words: Vec<&str> =
                    digits.chars().map(|d| DIGITS[(d as u8 - b'0') as usize]).collect();
                out.push_str(&words.join(" "));
            }
        }
    }
    out
}

/// A number in words ("two thousand five hundred", "fourteen"), up to 99,999.
fn number(n: u32) -> String {
    const ONES: [&str; 20] = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];
    const TENS: [&str; 10] =
        ["", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];
    let below_100 = |n: u32| -> String {
        if n < 20 {
            ONES[n as usize].to_string()
        } else if n.is_multiple_of(10) {
            TENS[(n / 10) as usize].to_string()
        } else {
            format!("{} {}", TENS[(n / 10) as usize], ONES[(n % 10) as usize])
        }
    };
    let mut parts = Vec::new();
    if n >= 1000 {
        parts.push(format!("{} thousand", below_100(n / 1000)));
    }
    if n % 1000 >= 100 {
        parts.push(format!("{} hundred", ONES[(n % 1000 / 100) as usize]));
    }
    if !n.is_multiple_of(100) || n == 0 {
        parts.push(below_100(n % 100));
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_as_on_the_radio() {
        assert_eq!(
            spoken("Gutersloh Tower, Viper 1-1, ready for departure, runway 09"),
            "Gutersloh Tower, Viper one one, ready for departure, runway zero niner"
        );
        assert_eq!(spoken("turn left heading 275"), "turn left heading two seven five");
        assert_eq!(spoken("climb and maintain 2,000"), "climb and maintain two thousand");
        assert_eq!(spoken("2,500"), "two thousand five hundred");
        assert_eq!(spoken("12 miles northeast"), "twelve miles northeast");
        assert_eq!(spoken("1 mile from touchdown"), "one mile from touchdown");
        assert_eq!(spoken("slightly above glidepath"), "slightly above glide path");
    }
}
