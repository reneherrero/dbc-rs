mod expect;
mod impls;
mod keyword;
mod parse;
mod skip;
mod take;

#[derive(Debug)]
pub struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
}
