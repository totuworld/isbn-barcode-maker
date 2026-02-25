/// EAN-13 encoding tables
/// L-codes (odd parity)
const L_CODES: [&str; 10] = [
    "0001101", "0011001", "0010011", "0111101", "0100011",
    "0110001", "0101111", "0111011", "0110111", "0001011",
];

/// G-codes (even parity)
const G_CODES: [&str; 10] = [
    "0100111", "0110011", "0011011", "0100001", "0011101",
    "0111001", "0000101", "0010001", "0001001", "0010111",
];

/// R-codes (right side)
const R_CODES: [&str; 10] = [
    "1110010", "1100110", "1101100", "1000010", "1011100",
    "1001110", "1010000", "1000100", "1001000", "1110100",
];

/// First digit encoding pattern (which L/G pattern to use for digits 2-7)
const FIRST_DIGIT_PATTERNS: [&str; 10] = [
    "LLLLLL", "LLGLGG", "LLGGLG", "LLGGGL", "LGLLGG",
    "LGGLLG", "LGGGLL", "LGLGLG", "LGLGGL", "LGGLGL",
];

/// EAN-5 (add-on) check digit patterns
const EAN5_PATTERNS: [&str; 10] = [
    "GGLLL", "GLGLL", "GLLGL", "GLLLG", "LGGLL",
    "LLGGL", "LLLGG", "LGLGL", "LGLLG", "LLGLG",
];

/// Validate ISBN-13 check digit
pub fn validate_isbn13(isbn: &str) -> bool {
    if isbn.len() != 13 || !isbn.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let digits: Vec<u32> = isbn.chars().map(|c| c.to_digit(10).unwrap()).collect();
    let sum: u32 = digits.iter().enumerate().map(|(i, &d)| {
        if i % 2 == 0 { d } else { d * 3 }
    }).sum();
    sum % 10 == 0
}

/// Encode EAN-13 barcode as a vector of bar widths
/// Returns (bars, human_readable_text)
pub fn encode_ean13(isbn: &str) -> Option<Vec<u8>> {
    if isbn.len() != 13 || !isbn.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let digits: Vec<usize> = isbn.chars().map(|c| c.to_digit(10).unwrap() as usize).collect();
    let pattern = FIRST_DIGIT_PATTERNS[digits[0]];

    let mut modules = String::new();

    // Start guard: 101
    modules.push_str("101");

    // Left side (digits 2-7, index 1-6)
    for (i, &ch) in pattern.as_bytes().iter().enumerate() {
        let digit = digits[i + 1];
        if ch == b'L' {
            modules.push_str(L_CODES[digit]);
        } else {
            modules.push_str(G_CODES[digit]);
        }
    }

    // Center guard: 01010
    modules.push_str("01010");

    // Right side (digits 8-13, index 7-12)
    for i in 7..13 {
        modules.push_str(R_CODES[digits[i]]);
    }

    // End guard: 101
    modules.push_str("101");

    Some(modules.bytes().map(|b| b - b'0').collect())
}

/// Calculate EAN-5 check digit and return encoding pattern
fn ean5_check(digits: &[usize; 5]) -> usize {
    let sum = digits[0] * 3 + digits[1] * 9 + digits[2] * 3 + digits[3] * 9 + digits[4] * 3;
    sum % 10
}

/// Encode EAN-5 add-on barcode
pub fn encode_ean5(addon: &str) -> Option<Vec<u8>> {
    if addon.len() != 5 || !addon.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let digits: [usize; 5] = [
        addon.chars().nth(0).unwrap().to_digit(10).unwrap() as usize,
        addon.chars().nth(1).unwrap().to_digit(10).unwrap() as usize,
        addon.chars().nth(2).unwrap().to_digit(10).unwrap() as usize,
        addon.chars().nth(3).unwrap().to_digit(10).unwrap() as usize,
        addon.chars().nth(4).unwrap().to_digit(10).unwrap() as usize,
    ];

    let check = ean5_check(&digits);
    let pattern = EAN5_PATTERNS[check];

    let mut modules = String::new();

    // Start: 1011
    modules.push_str("1011");

    for (i, &ch) in pattern.as_bytes().iter().enumerate() {
        let digit = digits[i];
        if ch == b'L' {
            modules.push_str(L_CODES[digit]);
        } else {
            modules.push_str(G_CODES[digit]);
        }
        // Separator between digits (not after last)
        if i < 4 {
            modules.push_str("01");
        }
    }

    Some(modules.bytes().map(|b| b - b'0').collect())
}
