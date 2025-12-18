use super::Message;

#[cfg(feature = "std")]
impl Message {
    #[must_use = "return value should be used"]
    pub fn to_dbc_string(&self) -> std::string::String {
        format!(
            "BO_ {} {} : {} {}",
            self.id(),
            self.name(),
            self.dlc(),
            self.sender()
        )
    }

    #[must_use = "return value should be used"]
    pub fn to_string_full(&self) -> std::string::String {
        let mut result = std::string::String::with_capacity(200 + (self.signals.len() * 100));
        result.push_str(&self.to_dbc_string());
        result.push('\n');

        for signal in self.signals().iter() {
            result.push_str(&signal.to_dbc_string());
            result.push('\n');
        }

        result
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_string_full())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Parser, Signal};

    #[test]
    fn test_message_to_dbc_string() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();
        let dbc_string = message.to_dbc_string();
        assert_eq!(dbc_string, "BO_ 256 EngineData : 8 ECM");
    }

    #[test]
    fn test_message_to_string_full() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
        )
        .unwrap();

        let message = Message::parse(&mut parser, &[signal1, signal2]).unwrap();
        let dbc_string = message.to_string_full();
        assert!(dbc_string.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(dbc_string.contains("SG_ RPM"));
        assert!(dbc_string.contains("SG_ Temp"));
    }

    #[test]
    fn test_message_to_dbc_string_empty_signals() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();

        let dbc_string = message.to_dbc_string();
        assert_eq!(dbc_string, "BO_ 256 EngineData : 8 ECM");

        let dbc_string_with_signals = message.to_string_full();
        assert_eq!(dbc_string_with_signals, "BO_ 256 EngineData : 8 ECM\n");
    }

    #[test]
    fn test_message_to_dbc_string_special_characters() {
        let data = b"BO_ 1234 Test_Message_With_Underscores : 4 Sender_Node";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();

        let dbc_string = message.to_dbc_string();
        assert_eq!(
            dbc_string,
            "BO_ 1234 Test_Message_With_Underscores : 4 Sender_Node"
        );
    }

    #[test]
    fn test_message_to_dbc_string_extended_id() {
        // Use a valid extended ID (max is 0x1FFF_FFFF = 536870911)
        let data = b"BO_ 536870911 ExtendedID : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();

        let dbc_string = message.to_dbc_string();
        assert_eq!(dbc_string, "BO_ 536870911 ExtendedID : 8 ECM");
    }

    #[test]
    fn test_message_to_dbc_string_dlc_edge_cases() {
        // Test DLC = 1
        let data = b"BO_ 256 MinDLC : 1 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();
        assert_eq!(message.to_dbc_string(), "BO_ 256 MinDLC : 1 ECM");

        // Test DLC = 64 (CAN FD max)
        let data2 = b"BO_ 257 MaxDLC : 64 ECM";
        let mut parser2 = Parser::new(data2).unwrap();
        let signals_empty: &[Signal] = &[];
        let message2 = Message::parse(&mut parser2, signals_empty).unwrap();
        assert_eq!(message2.to_dbc_string(), "BO_ 257 MaxDLC : 64 ECM");
    }

    #[test]
    fn test_message_display_trait() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Parse signal from DBC string instead of using builder
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *").unwrap(),
        )
        .unwrap();

        let message = Message::parse(&mut parser, &[signal]).unwrap();

        let display_str = format!("{}", message);
        assert!(display_str.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(display_str.contains("SG_ RPM"));
    }

    #[test]
    fn test_message_to_string_full_multiple() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Parse signals from DBC strings instead of using builders
        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *").unwrap(),
        )
        .unwrap();

        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\" *").unwrap(),
        )
        .unwrap();

        let message = Message::parse(&mut parser, &[signal1, signal2]).unwrap();

        let dbc_string = message.to_string_full();
        assert!(dbc_string.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(dbc_string.contains("SG_ RPM"));
        assert!(dbc_string.contains("SG_ Temp"));
        // Should have newlines between signals
        let lines: Vec<&str> = dbc_string.lines().collect();
        assert!(lines.len() >= 3); // Message line + at least 2 signal lines
    }

    #[test]
    fn test_message_can_2_0a_dlc_limits() {
        // CAN 2.0A: DLC can be 1-8 bytes (8-64 bits)
        // Test valid DLC values
        for dlc in 1..=8 {
            let data = format!("BO_ 256 EngineData : {} ECM", dlc);
            let mut parser = Parser::new(data.as_bytes()).unwrap();
            let signals: &[Signal] = &[];
            let message = Message::parse(&mut parser, signals).unwrap();
            assert_eq!(message.dlc(), dlc);
        }
    }

    #[test]
    fn test_message_can_2_0b_dlc_limits() {
        // CAN 2.0B: DLC can be 1-8 bytes (8-64 bits)
        // Test valid DLC values
        for dlc in 1..=8 {
            let data = format!("BO_ 256 EngineData : {} ECM", dlc);
            let mut parser = Parser::new(data.as_bytes()).unwrap();
            let signals: &[Signal] = &[];
            let message = Message::parse(&mut parser, signals).unwrap();
            assert_eq!(message.dlc(), dlc);
        }
    }

    #[test]
    fn test_message_can_fd_dlc_limits() {
        // CAN FD: DLC can be 1-64 bytes (8-512 bits)
        // Test valid DLC values up to 64
        for dlc in [1, 8, 12, 16, 20, 24, 32, 48, 64] {
            let data = format!("BO_ 256 EngineData : {} ECM", dlc);
            let mut parser = Parser::new(data.as_bytes()).unwrap();
            let signals: &[Signal] = &[];
            let message = Message::parse(&mut parser, signals).unwrap();
            assert_eq!(message.dlc(), dlc);
        }
    }

    #[test]
    fn test_message_signals_iterator_collect() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signal1 =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();
        let signal2 =
            Signal::parse(&mut Parser::new(b"SG_ Signal2 : 8|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();
        let signal3 =
            Signal::parse(&mut Parser::new(b"SG_ Signal3 : 16|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();

        let message = Message::parse(&mut parser, &[signal1, signal2, signal3]).unwrap();

        // Test that iterator can be used multiple times
        let names: Vec<&str> = message.signals().iter().map(|s| s.name()).collect();
        assert_eq!(names, vec!["Signal1", "Signal2", "Signal3"]);
    }
}
