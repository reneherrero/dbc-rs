mod core;
mod expect;
mod keyword;
mod parse;
mod skip;
mod take;

#[derive(Debug)]
pub struct Parser<'a> {
    pub(crate) input: &'a [u8],
    pub(crate) pos: usize,
    pub(crate) line: usize,
}
