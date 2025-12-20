use crate::{ByteOrder, ReceiversBuilder};

#[derive(Debug, Clone)]
pub struct SignalBuilder {
    pub(crate) name: Option<String>,
    pub(crate) start_bit: Option<u16>,
    pub(crate) length: Option<u16>,
    pub(crate) byte_order: Option<ByteOrder>,
    pub(crate) unsigned: Option<bool>,
    pub(crate) factor: Option<f64>,
    pub(crate) offset: Option<f64>,
    pub(crate) min: Option<f64>,
    pub(crate) max: Option<f64>,
    pub(crate) unit: Option<String>,
    pub(crate) receivers: ReceiversBuilder,
}

mod build;
mod impls;
