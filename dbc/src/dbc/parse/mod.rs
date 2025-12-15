mod ba;
mod ba_def;
mod ba_def_def;
mod ba_def_sgtype;
mod ba_sgtype;
mod bo;
mod bo_tx_bu;
mod bs;
mod bu;
mod cm;
mod envvar_data;
mod ev;
mod ev_data;
mod ns;
mod parse_state;
mod sg_mul_val;
mod sgtype;
mod sgtype_val;
mod sig_group;
mod sig_type_ref;
mod sig_valtype;
mod unimplemented;
mod val;
mod val_table;
mod version;

pub use parse_state::ParseState;

use super::super::Dbc;
use crate::{Error, Parser, Result};

impl Dbc {
    /// Parse a DBC file from a string slice
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc_content = r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 EngineData : 8 ECM
    ///  SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm""#;
    ///
    /// let dbc = Dbc::parse(dbc_content)?;
    /// assert_eq!(dbc.messages().len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn parse(data: &str) -> Result<Self> {
        let mut parser = Parser::new(data.as_bytes())?;
        let mut state = ParseState::new();

        loop {
            // Skip comments (lines starting with //)
            parser.skip_newlines_and_spaces();
            if parser.starts_with(b"//") {
                parser.skip_to_end_of_line();
                continue;
            }

            let keyword_result = parser.peek_next_keyword();
            let keyword = match keyword_result {
                Ok(kw) => kw,
                Err(Error::UnexpectedEof) => break,
                Err(Error::Expected(_)) => {
                    if parser.starts_with(b"//") {
                        parser.skip_to_end_of_line();
                        continue;
                    }
                    return Err(keyword_result.unwrap_err());
                }
                Err(e) => return Err(e),
            };

            // Save position after peek_next_keyword (which skips whitespace, so we're at the keyword)
            let pos_at_keyword = parser.pos();

            // Handle keyword
            handle_keyword(&mut parser, &mut state, keyword, pos_at_keyword, data)?;
        }

        // Build final Dbc from parse state
        state.build_dbc()
    }

    /// Parse a DBC file from a byte slice
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc_bytes = b"VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM";
    /// let dbc = Dbc::parse_bytes(dbc_bytes)?;
    /// println!("Parsed {} messages", dbc.messages().len());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn parse_bytes(data: &[u8]) -> Result<Dbc> {
        let content =
            core::str::from_utf8(data).map_err(|_e| Error::Expected(Error::INVALID_UTF8))?;
        Dbc::parse(content)
    }
}

fn handle_keyword(
    parser: &mut Parser,
    state: &mut ParseState,
    keyword: &str,
    pos_at_keyword: usize,
    data: &str,
) -> Result<()> {
    use crate::{
        BA_, BA_DEF_, BA_DEF_DEF_, BA_DEF_DEF_REL_, BA_DEF_REL_, BA_DEF_SGTYPE_, BA_REL_,
        BA_SGTYPE_, BO_, BO_TX_BU_, BS_, BU_, BU_BO_REL_, BU_EV_REL_, BU_SG_REL_, CAT_, CAT_DEF_,
        CM_, ENVVAR_DATA_, EV_, EV_DATA_, FILTER, NS_, NS_DESC_, SG_MUL_VAL_, SGTYPE_, SGTYPE_VAL_,
        SIG_GROUP_, SIG_TYPE_REF_, SIG_VALTYPE_, SIGTYPE_VALTYPE_, VAL_, VAL_TABLE_, VERSION,
    };

    match keyword {
        VERSION => version::handle_version(parser, state),
        BU_ => bu::handle_bu(parser, state, pos_at_keyword, data),
        BO_ => bo::handle_bo(parser, state, pos_at_keyword, data),
        NS_ => ns::handle_ns(parser),
        BA_DEF_ => ba_def::handle_ba_def(parser, state),
        BA_DEF_DEF_ => ba_def_def::handle_ba_def_def(parser, state),
        BA_ => ba::handle_ba(parser, state),
        BA_DEF_SGTYPE_ => ba_def_sgtype::handle_ba_def_sgtype(parser, state),
        BA_SGTYPE_ => ba_sgtype::handle_ba_sgtype(parser, state),
        VAL_ => val::handle_val(parser, state),
        VAL_TABLE_ => val_table::handle_val_table(parser, state),
        SGTYPE_ => sgtype::handle_sgtype(parser, state),
        SIG_TYPE_REF_ => sig_type_ref::handle_sig_type_ref(parser, state),
        SGTYPE_VAL_ => sgtype_val::handle_sgtype_val(parser, state),
        SIG_VALTYPE_ => sig_valtype::handle_sig_valtype(parser, state),
        SG_MUL_VAL_ => sg_mul_val::handle_sg_mul_val(parser, state),
        SIG_GROUP_ => sig_group::handle_sig_group(parser, state),
        BO_TX_BU_ => bo_tx_bu::handle_bo_tx_bu(parser, state),
        EV_ => ev::handle_ev(parser, state),
        ENVVAR_DATA_ => envvar_data::handle_envvar_data(parser, state),
        EV_DATA_ => ev_data::handle_ev_data(parser, state),
        BS_ => bs::handle_bs(parser, state),
        CM_ => cm::handle_cm(parser, state),
        BA_DEF_REL_ => unimplemented::handle_unimplemented(parser, keyword),
        BA_REL_ => unimplemented::handle_unimplemented(parser, keyword),
        BA_DEF_DEF_REL_ => unimplemented::handle_unimplemented(parser, keyword),
        BU_SG_REL_ => unimplemented::handle_unimplemented(parser, keyword),
        BU_EV_REL_ => unimplemented::handle_unimplemented(parser, keyword),
        BU_BO_REL_ => unimplemented::handle_unimplemented(parser, keyword),
        NS_DESC_ => unimplemented::handle_unimplemented(parser, keyword),
        SIGTYPE_VALTYPE_ => unimplemented::handle_unimplemented(parser, keyword),
        CAT_DEF_ => unimplemented::handle_unimplemented(parser, keyword),
        CAT_ => unimplemented::handle_unimplemented(parser, keyword),
        FILTER => unimplemented::handle_unimplemented(parser, keyword),
        _ if keyword == crate::VECTOR__INDEPENDENT_SIG_MSG || keyword == crate::VECTOR__XXX => {
            // These are identifiers used in signal definitions, not top-level keywords
            // If peek_next_keyword matches these, it's likely a parsing error
            // Skip and continue
            let _ = parser.expect(keyword.as_bytes()).ok();
            parser.skip_to_end_of_line();
            Ok(())
        }
        _ => {
            // Unknown keyword, skip
            parser.skip_to_end_of_line();
            Ok(())
        }
    }
}
