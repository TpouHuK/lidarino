use rppal_mcp23s17::{ChipSelect, HardwareAddress, Port, SpiBus, SpiMode};

pub type Mcp23s17 = Mcp23s17Mock;
pub struct Mcp23s17Mock {}

impl Mcp23s17Mock {
    pub fn new(
        _: HardwareAddress,
        _: SpiBus,
        _: ChipSelect,
        _: u32,
        _: SpiMode,
    ) -> Result<Self, ()> {
        Ok(Mcp23s17Mock {})
    }

    pub fn get(&self, _: Port, _: u8) -> Result<pin::OutputPin, ()> {
        Ok(pin::OutputPin {})
    }
}

pub mod pin {
    pub struct OutputPin {}
    impl OutputPin {
        pub fn set_low(&self) -> Result<(), ()> {
            Ok(())
        }
        pub fn set_high(&self) -> Result<(), ()> {
            Ok(())
        }
        pub fn into_output_pin_low(self) -> Result<Self, ()> {
            Ok(self)
        }
    }
}
