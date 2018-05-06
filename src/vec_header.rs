#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use errors::*;
use HeaderField;
use HeaderFieldCollection;

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
