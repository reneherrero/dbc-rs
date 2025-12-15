use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::{EnvironmentVariable, EnvironmentVariableAccessType, EnvironmentVariableType, Error};
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_ev(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Parse EV_ env_var_name : env_var_type [minimum|maximum] unit initial_value ev_id access_type access_node {, access_node} ;
        // Example: EV_ TestVar : 0 [0|100] "" 0 0 DUMMY_NODE_VECTOR0 ECM,TCM ;
        if parser.expect(crate::EV_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        if let Ok(env_var) = (|| -> Result<EnvironmentVariable> {
            // Parse env_var_name (identifier)
            let name = parser.parse_identifier()?;
            let name = crate::validate_name(name)?;
            let name = name.as_str().to_string();
            parser.skip_newlines_and_spaces();

            // Expect colon
            parser
                .expect(b":")
                .map_err(|_| Error::Expected("Expected ':' after env var name"))?;
            parser.skip_newlines_and_spaces();

            // Parse env_var_type (0, 1, or 2)
            let var_type_num = parser.parse_u32()?;
            let var_type = match var_type_num {
                0 => EnvironmentVariableType::Integer,
                1 => EnvironmentVariableType::Float,
                2 => EnvironmentVariableType::String,
                _ => {
                    return Err(Error::Expected("Expected env_var_type (0, 1, or 2)"));
                }
            };
            parser.skip_newlines_and_spaces();

            // Parse [minimum|maximum]
            parser.expect(b"[").map_err(|_| Error::Expected("Expected '['"))?;
            parser.skip_newlines_and_spaces();
            let minimum = parser
                .parse_f64()
                .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
            parser.skip_newlines_and_spaces();
            parser.expect(b"|").map_err(|_| Error::Expected("Expected '|'"))?;
            parser.skip_newlines_and_spaces();
            let maximum = parser
                .parse_f64()
                .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
            parser.skip_newlines_and_spaces();
            parser.expect(b"]").map_err(|_| Error::Expected("Expected ']'"))?;
            parser.skip_newlines_and_spaces();

            // Parse unit (quoted string)
            let unit = if parser.expect(b"\"").is_ok() {
                let unit_bytes = parser
                    .take_until_quote(false, crate::MAX_NAME_SIZE)
                    .map_err(|_| Error::Expected("Expected closing quote"))?;
                core::str::from_utf8(unit_bytes)
                    .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?
                    .to_string()
            } else {
                String::new()
            };
            parser.skip_newlines_and_spaces();

            // Parse initial_value (double)
            let initial_value = parser
                .parse_f64()
                .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
            parser.skip_newlines_and_spaces();

            // Parse ev_id (unsigned_integer, obsolete)
            let ev_id = parser
                .parse_u32()
                .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
            parser.skip_newlines_and_spaces();

            // Parse access_type: DUMMY_NODE_VECTOR0-3 or DUMMY_NODE_VECTOR8000-8003
            let access_type = if parser.starts_with(b"DUMMY_NODE_VECTOR") {
                parser.expect(b"DUMMY_NODE_VECTOR").ok();
                let access_num = parser
                    .parse_u32()
                    .map_err(|_| Error::Expected("Expected access type number"))?;

                if (8000..=8003).contains(&access_num) {
                    EnvironmentVariableAccessType::StringType((access_num - 8000) as u16)
                } else {
                    match access_num {
                        0 => EnvironmentVariableAccessType::Unrestricted,
                        1 => EnvironmentVariableAccessType::ReadOnly,
                        2 => EnvironmentVariableAccessType::WriteOnly,
                        3 => EnvironmentVariableAccessType::ReadWrite,
                        _ => return Err(Error::Expected("Invalid access type")),
                    }
                }
            } else {
                return Err(Error::Expected("Expected DUMMY_NODE_VECTOR"));
            };
            parser.skip_newlines_and_spaces();

            // Parse access_node list (comma or space separated)
            let mut access_nodes = std::vec::Vec::new();
            loop {
                // Check for semicolon (end)
                if parser.starts_with(b";") {
                    parser.expect(b";").ok();
                    break;
                }

                // Try to parse identifier (node name or VECTOR__XXX)
                if parser.starts_with(crate::VECTOR__XXX.as_bytes()) {
                    parser.expect(crate::VECTOR__XXX.as_bytes()).ok();
                    access_nodes.push(crate::VECTOR__XXX.to_string());
                } else if let Ok(node_name) = parser.parse_identifier() {
                    let node_name_validated = crate::validate_name(node_name)?;
                    access_nodes.push(node_name_validated.as_str().to_string());
                } else {
                    break;
                }

                parser.skip_newlines_and_spaces();

                // Check for comma (more nodes)
                if parser.expect(b",").is_ok() {
                    parser.skip_newlines_and_spaces();
                    continue;
                }

                // Check for semicolon or end
                if parser.starts_with(b";") || parser.peek_byte_at(0).is_none() {
                    if parser.starts_with(b";") {
                        parser.expect(b";").ok();
                    }
                    break;
                }
            }

            Ok(EnvironmentVariable::new(
                name,
                var_type,
                minimum,
                maximum,
                unit,
                initial_value,
                ev_id,
                access_type,
                access_nodes,
            ))
        })() {
            state.environment_variables_buffer.push(env_var);
        } else {
            // If parsing fails, just skip the line
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume EV_ keyword and skip the rest
        let _ = parser.expect(crate::EV_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
