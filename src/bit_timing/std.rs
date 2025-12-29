use super::BitTiming;
use std::fmt::{Display, Formatter, Result};

impl Display for BitTiming {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match (self.baudrate, self.btr1, self.btr2) {
            (Some(baudrate), Some(btr1), Some(btr2)) => {
                write!(f, "BS_: {} : {},{}", baudrate, btr1, btr2)
            }
            (Some(baudrate), _, _) => {
                write!(f, "BS_: {}", baudrate)
            }
            _ => {
                write!(f, "BS_:")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_empty() {
        let bt = BitTiming::new();
        assert_eq!(bt.to_string(), "BS_:");
    }

    #[test]
    fn test_display_baudrate_only() {
        let bt = BitTiming::with_baudrate(500);
        assert_eq!(bt.to_string(), "BS_: 500");
    }

    #[test]
    fn test_display_full() {
        let bt = BitTiming::with_btr(500, 12, 34);
        assert_eq!(bt.to_string(), "BS_: 500 : 12,34");
    }
}
