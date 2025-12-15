use super::Signal;
use crate::{ByteOrder, Receivers};

impl Signal {
    #[cfg(feature = "std")]
    #[must_use]
    pub fn to_dbc_string(&self) -> std::string::String {
        let mut result = std::string::String::with_capacity(100); // Pre-allocate reasonable capacity

        result.push_str(" SG_ ");
        result.push_str(self.name());
        result.push_str(" : ");
        result.push_str(&self.start_bit().to_string());
        result.push('|');
        result.push_str(&self.length().to_string());
        result.push('@');

        // Byte order: 0 for BigEndian (Motorola), 1 for LittleEndian (Intel)
        // Per Vector DBC spec v1.0.1: "Big endian is stored as '0', little endian is stored as '1'"
        match self.byte_order() {
            ByteOrder::BigEndian => result.push('0'), // @0 = Big Endian (Motorola)
            ByteOrder::LittleEndian => result.push('1'), // @1 = Little Endian (Intel)
        }

        // Sign: + for unsigned, - for signed
        if self.is_unsigned() {
            result.push('+');
        } else {
            result.push('-');
        }

        // Factor and offset: (factor,offset)
        result.push_str(" (");
        use core::fmt::Write;
        write!(result, "{}", self.factor()).unwrap();
        result.push(',');
        write!(result, "{}", self.offset()).unwrap();
        result.push(')');

        // Min and max: [min|max]
        result.push_str(" [");
        write!(result, "{}", self.min()).unwrap();
        result.push('|');
        write!(result, "{}", self.max()).unwrap();
        result.push(']');

        // Unit: "unit" or ""
        result.push(' ');
        if let Some(unit) = self.unit() {
            result.push('"');
            result.push_str(unit);
            result.push('"');
        } else {
            result.push_str("\"\"");
        }

        // Receivers: * for Broadcast, space-separated list for Nodes, or empty
        match self.receivers() {
            Receivers::Broadcast => {
                result.push(' ');
                result.push('*');
            }
            Receivers::Nodes(nodes) => {
                if !nodes.is_empty() {
                    result.push(' ');
                    for (i, node) in self.receivers().iter().enumerate() {
                        if i > 0 {
                            result.push(' ');
                        }
                        result.push_str(node.as_str());
                    }
                }
            }
            Receivers::None => {
                // No receivers specified - nothing to add
            }
        }

        result
    }
}
