pub mod es8388;
pub mod get_clock_ntp;
pub mod peripheral;
mod psram;

pub mod share_i2c_bus {
    use core::cell::RefCell;
    use embedded_hal::i2c::{ErrorType, I2c, Operation};
    use std::rc::Rc;

    pub struct SharedI2cDevice<T>(pub Rc<RefCell<T>>);

    impl<T: ErrorType> ErrorType for SharedI2cDevice<T> {
        type Error = T::Error;
    }

    impl<T: I2c> I2c for SharedI2cDevice<T> {
        fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
            self.0.borrow_mut().read(address, read)
        }

        fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
            self.0.borrow_mut().write(address, write)
        }

        fn write_read(
            &mut self,
            address: u8,
            write: &[u8],
            read: &mut [u8],
        ) -> Result<(), Self::Error> {
            self.0.borrow_mut().write_read(address, write, read)
        }

        fn transaction(
            &mut self,
            address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            self.0.borrow_mut().transaction(address, operations)
        }
    }
}
