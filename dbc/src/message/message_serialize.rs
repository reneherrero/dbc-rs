#[cfg(feature = "std")]
use super::Message;

#[cfg(feature = "std")]
impl Message {
    #[must_use]
    pub fn to_dbc_string(&self) -> std::string::String {
        format!(
            "BO_ {} {} : {} {}",
            self.id(),
            self.name(),
            self.dlc(),
            self.sender()
        )
    }

    #[must_use]
    pub fn to_string_full(&self) -> std::string::String {
        let mut result = std::string::String::with_capacity(200 + (self.signals().len() * 100));
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
