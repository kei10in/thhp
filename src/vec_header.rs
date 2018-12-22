#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::errors::*;
use crate::HeaderField;
use crate::HeaderFieldCollection;

/// A basic implementation of `HeaderFieldCollection`.
impl<'buffer> HeaderFieldCollection<'buffer> for Vec<HeaderField<'buffer>> {
    fn push(&mut self, header_field: HeaderField<'buffer>) -> Result<()> {
        if self.len() == self.capacity() {
            Err(OutOfCapacity.into())
        } else {
            self.push(header_field);
            Ok(())
        }
    }
}
