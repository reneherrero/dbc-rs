use crate::SignalBuilder;

#[derive(Debug)]
pub struct MessageBuilder {
    pub(crate) id: Option<u32>,
    pub(crate) name: Option<String>,
    pub(crate) dlc: Option<u8>,
    pub(crate) sender: Option<String>,
    pub(crate) signals: Vec<SignalBuilder>,
    pub(crate) comment: Option<String>,
}

mod build;
mod impls;
