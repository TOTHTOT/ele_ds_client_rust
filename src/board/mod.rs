pub mod button;
pub mod es8388;
pub mod get_clock_ntp;
pub mod peripheral;
pub mod power_manage;
pub mod psram;

pub mod share_i2c_bus {
    use core::cell::RefCell;
    use embedded_hal::i2c::{ErrorType, I2c, Operation};
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    pub trait I2cBusHandle {
        type Bus;
        fn bus<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&mut Self::Bus) -> R;
    }

    impl<T> I2cBusHandle for Arc<Mutex<T>> {
        type Bus = T;

        fn bus<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&mut Self::Bus) -> R,
        {
            f(&mut self.lock().expect("I2C Lock poisoned"))
        }
    }

    impl<T> I2cBusHandle for Rc<RefCell<T>> {
        type Bus = T;

        fn bus<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&mut Self::Bus) -> R,
        {
            f(&mut self.borrow_mut())
        }
    }

    pub struct SharedI2cDevice<H>(pub H);

    impl<H> ErrorType for SharedI2cDevice<H>
    where
        H: I2cBusHandle,
        H::Bus: ErrorType,
    {
        type Error = <H::Bus as ErrorType>::Error;
    }

    impl<H> I2c for SharedI2cDevice<H>
    where
        H: I2cBusHandle,
        H::Bus: I2c,
    {
        fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
            self.0.bus(|bus| bus.read(address, read))
        }

        fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
            self.0.bus(|bus| bus.write(address, write))
        }

        fn write_read(
            &mut self,
            address: u8,
            write: &[u8],
            read: &mut [u8],
        ) -> Result<(), Self::Error> {
            self.0.bus(|bus| bus.write_read(address, write, read))
        }

        fn transaction(
            &mut self,
            address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            self.0.bus(|bus| bus.transaction(address, operations))
        }
    }
}
