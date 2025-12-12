use crate::compat::Vec;
use crate::{MAX_SIGNALS_PER_MESSAGE, Signal};

/// Encapsulates the signals array for a message
///
/// Uses `Vec<Signal>` for dynamic sizing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignalList {
    signals: Vec<Signal, { MAX_SIGNALS_PER_MESSAGE }>,
}

impl From<&[Signal]> for SignalList {
    fn from(signals: &[Signal]) -> Self {
        Self::from_slice(signals)
    }
}

#[cfg(feature = "std")]
impl From<std::vec::Vec<Signal>> for SignalList {
    fn from(signals: std::vec::Vec<Signal>) -> Self {
        Self::from_slice(&signals)
    }
}

impl From<Vec<Signal, { MAX_SIGNALS_PER_MESSAGE }>> for SignalList {
    fn from(signals: Vec<Signal, { MAX_SIGNALS_PER_MESSAGE }>) -> Self {
        Self::from_slice(signals.as_slice())
    }
}

impl SignalList {
    /// Create SignalList from a slice of signals by cloning them
    pub(crate) fn from_slice(signals: &[Signal]) -> Self {
        let count = signals.len().min(MAX_SIGNALS_PER_MESSAGE);
        let signals_vec: Vec<Signal, { MAX_SIGNALS_PER_MESSAGE }> =
            signals.iter().take(count).cloned().collect();
        Self {
            signals: signals_vec,
        }
    }

    /// Get an iterator over the signals
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"")?;
    /// let message = dbc.messages().at(0).unwrap();
    /// for signal in message.signals().iter() {
    ///     println!("Signal: {} (start: {}, length: {})", signal.name(), signal.start_bit(), signal.length());
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter(&self) -> impl Iterator<Item = &Signal> + '_ {
        self.signals.iter()
    }

    /// Get the number of signals
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"")?;
    /// let message = dbc.messages().at(0).unwrap();
    /// assert_eq!(message.signals().len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.signals.len()
    }

    /// Returns `true` if there are no signals
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let message = dbc.messages().at(0).unwrap();
    /// assert!(message.signals().is_empty());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a signal by index, or None if index is out of bounds
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"")?;
    /// let message = dbc.messages().at(0).unwrap();
    /// if let Some(signal) = message.signals().at(0) {
    ///     assert_eq!(signal.name(), "RPM");
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn at(&self, index: usize) -> Option<&Signal> {
        self.signals.get(index)
    }

    /// Find a signal by name, or None if not found
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"")?;
    /// let message = dbc.messages().at(0).unwrap();
    /// if let Some(signal) = message.signals().find("RPM") {
    ///     assert_eq!(signal.name(), "RPM");
    ///     assert_eq!(signal.factor(), 0.25);
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&Signal> {
        self.iter().find(|s| s.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::SignalList;
    use crate::{Parser, Signal};

    // Tests that require std feature (for from_slice)
    #[cfg(feature = "std")]
    mod tests_with_std {
        use super::*;

        #[test]
        fn test_signals_from_slice() {
            let signal1 = Signal::parse(
                &mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap(),
            )
            .unwrap();
            let signal2 = Signal::parse(
                &mut Parser::new(b"SG_ Signal2 : 8|8@0+ (1,0) [0|255] \"\"").unwrap(),
            )
            .unwrap();

            let signals = SignalList::from_slice(&[signal1, signal2]);
            assert_eq!(signals.len(), 2);
            assert!(!signals.is_empty());
            assert_eq!(signals.at(0).unwrap().name(), "Signal1");
            assert_eq!(signals.at(1).unwrap().name(), "Signal2");
        }

        #[test]
        fn test_signals_from_signals_slice_empty() {
            let signals = SignalList::from_slice(&[]);
            assert_eq!(signals.len(), 0);
            assert!(signals.is_empty());
            assert!(signals.at(0).is_none());
        }

        #[test]
        fn test_signals_from_slice_multiple() {
            // Test with multiple signals to verify capacity handling
            let signal1 = Signal::parse(
                &mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap(),
            )
            .unwrap();
            let signal2 = Signal::parse(
                &mut Parser::new(b"SG_ Signal2 : 8|8@0+ (1,0) [0|255] \"\"").unwrap(),
            )
            .unwrap();
            let signal3 = Signal::parse(
                &mut Parser::new(b"SG_ Signal3 : 16|8@0+ (1,0) [0|255] \"\"").unwrap(),
            )
            .unwrap();

            let signals = SignalList::from_slice(&[signal1, signal2, signal3]);
            assert_eq!(signals.len(), 3);
            assert_eq!(signals.at(0).unwrap().name(), "Signal1");
            assert_eq!(signals.at(1).unwrap().name(), "Signal2");
            assert_eq!(signals.at(2).unwrap().name(), "Signal3");
        }
    }

    #[test]
    fn test_signals_find_not_found() {
        let signal1 =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();

        let signals = SignalList::from_slice(&[signal1]);
        assert!(signals.find("Nonexistent").is_none());
        assert!(signals.find("").is_none());
        assert!(signals.find("signal1").is_none()); // Case sensitive
    }

    #[test]
    fn test_signals_find_first_match() {
        let signal1 =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();
        let signal2 =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 8|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap(); // Same name (shouldn't happen in practice but test the behavior)

        let signals = SignalList::from_slice(&[signal1, signal2]);
        // Should find the first match
        let found = signals.find("Signal1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().start_bit(), 0); // First signal
    }
}
