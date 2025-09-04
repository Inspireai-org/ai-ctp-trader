// SPI 实现模块
// 包含行情和交易的 SPI 回调处理

pub mod md_spi;
pub mod trader_spi;

pub use md_spi::MdSpiImpl;
pub use trader_spi::TraderSpiImpl;