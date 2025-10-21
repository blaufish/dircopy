// Convert "128K" into 128*1024, and such
pub fn s2i(string: String) -> usize {
    let mut prefix: usize = 0;
    let mut exponent: usize = 1;
    for c in string.chars() {
        match c {
            'K' => exponent = 1024,
            'M' => exponent = 1024 * 1024,
            'G' => exponent = 1024 * 1024 * 1024,
            '0' => prefix = prefix * 10,
            '1' => prefix = prefix * 10 + 1,
            '2' => prefix = prefix * 10 + 2,
            '3' => prefix = prefix * 10 + 3,
            '4' => prefix = prefix * 10 + 4,
            '5' => prefix = prefix * 10 + 5,
            '6' => prefix = prefix * 10 + 6,
            '7' => prefix = prefix * 10 + 7,
            '8' => prefix = prefix * 10 + 8,
            '9' => prefix = prefix * 10 + 9,
            _ => eprintln!("Unable to parse: {}", string),
        }
    }
    let result = prefix * exponent;
    if result < 1 {
        eprintln!("Unable to parse: {}", string)
    }
    return result;
}

pub fn bandwidth(read_bytes: usize, seconds: u64) -> String {
    if seconds == 0 {
        return String::from("NaN");
    }
    let mut rb = (read_bytes as f64) / (seconds as f64);
    let sufixes: Vec<&str> = vec!["B", "KB", "MB", "GB", "TB", "PB"];
    let mut suff = "";
    for s in sufixes {
        suff = s;
        if rb < 1000.0 {
            break;
        }
        rb = rb / 1000.0;
    }
    return format!("{:.3} {}/s", rb, suff);
}
