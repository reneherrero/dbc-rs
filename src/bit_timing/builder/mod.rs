/// Builder for creating `BitTiming` programmatically.
///
/// This builder allows you to construct bit timing configuration when building
/// DBC files programmatically.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::BitTimingBuilder;
///
/// // Empty bit timing (most common)
/// let bt = BitTimingBuilder::new().build()?;
/// assert!(bt.is_empty());
///
/// // With baudrate only
/// let bt = BitTimingBuilder::new()
///     .baudrate(500000)
///     .build()?;
/// assert_eq!(bt.baudrate(), Some(500000));
///
/// // With full timing parameters
/// let bt = BitTimingBuilder::new()
///     .baudrate(500000)
///     .btr1(1)
///     .btr2(2)
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Feature Requirements
///
/// This builder requires the `std` feature to be enabled.
#[derive(Debug, Clone)]
pub struct BitTimingBuilder {
    baudrate: Option<u32>,
    btr1: Option<u32>,
    btr2: Option<u32>,
}

mod build;
mod impls;
