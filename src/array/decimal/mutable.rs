use std::any::Any;
use std::sync::Arc;
use crate::array::{Array, DecimalArray,  MutableArray, MutableFixedSizeBinaryArray, TryPush};
use crate::bitmap::{ MutableBitmap};
use crate::datatypes::DataType;
use crate::error::{ArrowError, Result};

///
/// Array Builder for [`DecimalArray`]
///
/// See [`DecimalArray`] for example.
///
#[derive(Debug)]
pub struct MutableDecimalArray {
    inner: MutableFixedSizeBinaryArray,
    precision: usize,
    scale: usize,
}

impl MutableArray for MutableDecimalArray {
    fn data_type(&self) -> &DataType {
        &DataType::Decimal(self.precision, self.scale)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn validity(&self) -> Option<&MutableBitmap> {
        self.inner.validity()
    }

    fn as_box(&mut self) -> Box<dyn Array> {
        Box::new(DecimalArray::from_data(
            self.precision,
            self.scale,
            self.inner.as_fixed_size_array(),
        ))
    }

    fn as_arc(&mut self) -> Arc<dyn Array> {
        Arc::new(DecimalArray::from_data(
            self.precision,
            self.scale,
            self.inner.as_fixed_size_array(),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn push_null(&mut self) {
        self.inner.push::<&[u8]>(None)
    }

    fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }
}

impl MutableDecimalArray {
    /// Creates a new `BinaryBuilder`, `capacity` is the number of bytes in the values
    /// array
    pub fn new(capacity: usize, precision: usize, scale: usize) -> Self {
        let byte_width = 16;
        Self {
            inner: MutableFixedSizeBinaryArray::with_capacity(byte_width, capacity),
            precision,
            scale,
        }
    }

    fn from_i128_to_fixed_size_bytes(v: i128, size: usize) -> Result<Vec<u8>> {
        if size > 16 {
            return Err(ArrowError::InvalidArgumentError(
                "DecimalBuilder only supports values up to 16 bytes.".to_string(),
            ));
        }
        let res = v.to_le_bytes();
        let start_byte = 16 - size;
        Ok(res[start_byte..16].to_vec())
    }

}

impl TryPush<Option<i128>> for MutableDecimalArray {
    fn try_push(&mut self, value: Option<i128>) -> Result<()> {
        match value {
            Some(value) => {
                if value > super::MAX_DECIMAL_FOR_EACH_PRECISION[self.precision - 1]
                    || value < super::MIN_DECIMAL_FOR_EACH_PRECISION[self.precision - 1]
                {
                    return Err(ArrowError::InvalidArgumentError(format!(
                        "The value of {} i128 is not compatible with Decimal({},{})",
                        value, self.precision, self.scale
                    )));
                }
                let value_as_bytes = Self::from_i128_to_fixed_size_bytes(
                    value,
                    self.inner.size(),
                )?;
                if self.inner.size() != value_as_bytes.len() {
                    return Err(ArrowError::InvalidArgumentError(
                        "Byte slice does not have the same length as DecimalBuilder value lengths".to_string()
                    ));
                }
                self.inner.try_push(Some(value_as_bytes));
            }
            None => {
                self.inner.try_push::<&[u8]>(None);
            }
        }
        Ok(())
    }
}
