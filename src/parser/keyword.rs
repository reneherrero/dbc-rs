use super::Parser;
use crate::Error;

// Keywords grouped by first character for fast dispatch
// Within each group, longer keywords come first to avoid prefix matching issues

// 'B' keywords (most common: BO_, BU_, BS_, BA_*)
const KEYWORDS_B: &[&str] = &[
    "BU_BO_REL_",
    "BU_EV_REL_",
    "BU_SG_REL_",
    "BA_DEF_DEF_REL_",
    "BA_DEF_DEF_",
    "BA_DEF_REL_",
    "BA_DEF_SGTYPE_",
    "BA_SGTYPE_",
    "BA_DEF_",
    "BO_TX_BU_",
    "BA_REL_",
    "BA_",
    "BS_",
    "BU_",
    "BO_",
];

// 'S' keywords
const KEYWORDS_S: &[&str] = &[
    "SIGTYPE_VALTYPE_",
    "SIG_TYPE_REF_",
    "SIG_VALTYPE_",
    "SG_MUL_VAL_",
    "SIG_GROUP_",
    "SGTYPE_VAL_",
    "SG_",
];

// 'V' keywords
const KEYWORDS_V: &[&str] = &[
    "VECTOR__INDEPENDENT_SIG_MSG",
    "Vector__XXX",
    "VAL_TABLE_",
    "VERSION",
    "VAL_",
];

// 'C' keywords
const KEYWORDS_C: &[&str] = &["CAT_DEF_", "CAT_", "CM_"];

// 'E' keywords
const KEYWORDS_E: &[&str] = &["ENVVAR_DATA_", "EV_DATA_", "EV_"];

// 'N' keywords
const KEYWORDS_N: &[&str] = &["NS_DESC_", "NS_"];

// 'F' keywords
const KEYWORDS_F: &[&str] = &["FILTER"];

impl<'a> Parser<'a> {
    pub fn peek_next_keyword(&mut self) -> crate::Result<&'a str> {
        // Skip newlines and spaces to find the next keyword
        self.skip_newlines_and_spaces();

        // Check if we're at EOF
        if self.eof() {
            return Err(self.err_unexpected_eof());
        }

        // Get first byte for dispatch
        let first_byte = match self.peek_byte_at(0) {
            Some(b) => b,
            None => return Err(self.err_unexpected_eof()),
        };

        // Dispatch to appropriate keyword group based on first character
        let keywords: &[&str] = match first_byte {
            b'B' => KEYWORDS_B,
            b'S' => KEYWORDS_S,
            b'V' => KEYWORDS_V,
            b'C' => KEYWORDS_C,
            b'E' => KEYWORDS_E,
            b'N' => KEYWORDS_N,
            b'F' => KEYWORDS_F,
            _ => return Err(self.err_expected(Error::EXPECTED_KEYWORD)),
        };

        // Try to match each keyword in the group (longer ones first)
        for keyword in keywords {
            let keyword_bytes = keyword.as_bytes();
            if self.starts_with(keyword_bytes) {
                // Check if the character after the keyword is a valid delimiter
                let next_byte = self.peek_byte_at(keyword_bytes.len());
                let is_valid_delimiter = next_byte
                    .map(|b| matches!(b, b' ' | b'\t' | b':' | b'\n' | b'\r'))
                    .unwrap_or(true); // EOF is valid

                if is_valid_delimiter {
                    return Ok(keyword);
                }
            }
        }

        // No keyword matched
        Err(self.err_expected(Error::EXPECTED_KEYWORD))
    }
}
