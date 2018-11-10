// In iNES mapper number order.

// #1 NROM
mod nrom;
pub use self::nrom::NROM;

// #2 UxROM
mod uxrom;
pub use self::uxrom::UXROM;

// #3 CNROM
mod cnrom;
pub use self::cnrom::CNROM;

// #4 MMC1
mod mmc1;
pub use self::mmc1::MMC1;
