mod mutable;
mod iterator;

use std::convert::TryInto;
pub use mutable::MutableDecimalArray;
use crate::array::{Array, display_fmt, FixedSizeBinaryArray, FixedSizeListArray};
use crate::bitmap::Bitmap;
use crate::datatypes::DataType;

pub const MAX_DECIMAL_FOR_EACH_PRECISION: [i128; 38] = [
    9,
    99,
    999,
    9999,
    99999,
    999999,
    9999999,
    99999999,
    999999999,
    9999999999,
    99999999999,
    999999999999,
    9999999999999,
    99999999999999,
    999999999999999,
    9999999999999999,
    99999999999999999,
    999999999999999999,
    9999999999999999999,
    99999999999999999999,
    999999999999999999999,
    9999999999999999999999,
    99999999999999999999999,
    999999999999999999999999,
    9999999999999999999999999,
    99999999999999999999999999,
    999999999999999999999999999,
    9999999999999999999999999999,
    99999999999999999999999999999,
    999999999999999999999999999999,
    9999999999999999999999999999999,
    99999999999999999999999999999999,
    999999999999999999999999999999999,
    9999999999999999999999999999999999,
    99999999999999999999999999999999999,
    999999999999999999999999999999999999,
    9999999999999999999999999999999999999,
    170141183460469231731687303715884105727,
];
pub const MIN_DECIMAL_FOR_EACH_PRECISION: [i128; 38] = [
    -9,
    -99,
    -999,
    -9999,
    -99999,
    -999999,
    -9999999,
    -99999999,
    -999999999,
    -9999999999,
    -99999999999,
    -999999999999,
    -9999999999999,
    -99999999999999,
    -999999999999999,
    -9999999999999999,
    -99999999999999999,
    -999999999999999999,
    -9999999999999999999,
    -99999999999999999999,
    -999999999999999999999,
    -9999999999999999999999,
    -99999999999999999999999,
    -999999999999999999999999,
    -9999999999999999999999999,
    -99999999999999999999999999,
    -999999999999999999999999999,
    -9999999999999999999999999999,
    -99999999999999999999999999999,
    -999999999999999999999999999999,
    -9999999999999999999999999999999,
    -99999999999999999999999999999999,
    -999999999999999999999999999999999,
    -9999999999999999999999999999999999,
    -99999999999999999999999999999999999,
    -999999999999999999999999999999999999,
    -9999999999999999999999999999999999999,
    -170141183460469231731687303715884105728,
];

const DEFAULT_DECIMAL_LENGTH: usize = 16;

/// A [`DecimalArray`] is arrow's equivalent of an immutable `Vec<Option<Decimal>>`.
/// Cloning and slicing this struct is `O(1)`.
/// # Example
/// ```
/// use arrow2::array::DecimalArray;
/// # fn main() {
/// let array = DecimalArray::from_data(10, 2, [Some(1000), None, Some(100)]);
/// assert_eq!(array.value(0), 1000);
/// assert_eq!(array.values().as_slice(), b100.as_ref());
/// assert_eq!(array.offsets().as_slice(), &[0, 2, 2, 2 + 5]);
/// # }
/// ```
#[derive(Clone)]
pub struct DecimalArray {
    data_type: DataType,
    data: FixedSizeBinaryArray,
    precision: usize,
    scale: usize,
}

impl DecimalArray {
    fn default_data_data_type() -> DataType {
        DataType::FixedSizeBinary(DEFAULT_DECIMAL_LENGTH)
    }

    /// Returns a new empty [`DecimalArray`]
    #[inline]
    pub fn new_empty(precision: usize, scale: usize) -> Self {
        Self::from_data(precision, scale, FixedSizeBinaryArray::new_empty(Self::default_data_data_type()))
    }

    /// Returns a new empty [`DecimalArray`] whose all slots are null / `None`.
    #[inline]
    pub fn new_null(precision: usize, scale: usize, length: usize) -> Self {
        Self::from_data(precision, scale, FixedSizeBinaryArray::new_null(Self::default_data_data_type(), length))
    }

    /// Returns a new [`DecimalArray`]
    #[inline]
    pub fn from_data(precision: usize, scale: usize, data: FixedSizeBinaryArray) -> Self {
        Self {
            data_type: DataType::Decimal(precision, scale),
            scale, precision, data,
        }
    }

    /// Returns a slice of this [`DecimalArray`].
    /// # Implementation
    /// This operation is `O(1)` as it amounts to increase 3 ref counts.
    /// # Panics
    /// panics iff `offset + length > self.len()`
    pub fn slice(&self, offset: usize, length: usize) -> Self {
        assert!(
            offset + length <= self.len(),
            "the offset of the new Buffer cannot exceed the existing length"
        );
        unsafe { self.slice_unchecked(offset, length) }
    }
    /// Returns a slice of this [`DecimalArray`].
    /// # Implementation
    /// This operation is `O(1)` as it amounts to increase 3 ref counts.
    /// # Safety
    /// The caller must ensure that `offset + length <= self.len()`.
    pub unsafe fn slice_unchecked(&self, offset: usize, length: usize) -> Self {
        Self {
            data_type: self.data_type.clone(),
            precision: self.precision,
            scale: self.scale,
            data: self.data.slice_unchecked(offset, length)
        }
    }

    /// Sets the validity bitmap on this [`DecimalArray`].
    /// # Panic
    /// This function panics iff `validity.len() != self.len()`.
    pub fn with_validity(&self, validity: Option<Bitmap>) -> Self {
        if matches!(&validity, Some(bitmap) if bitmap.len() != self.len()) {
            panic!("validity should be as least as large as the array")
        }
        let mut arr = self.clone();
        arr.data = arr.data.with_validity(validity);
        arr
    }
}

impl DecimalArray {
    /// Returns the element at index `i` as i128.
    pub fn value(&self, i: usize) -> i128 {
        assert!(i < self.data.len(), "DecimalArray out of bounds access");
        let v = self.data.value(i);
        let bytes: [u8; DEFAULT_DECIMAL_LENGTH] = v.try_into().expect("DecimalArray elements are not 128bit integers.");
        i128::from_le_bytes(bytes)
    }

    pub fn precision(&self) -> usize {
        self.precision
    }

    pub fn scale(&self) -> usize {
        self.scale
    }
}

impl Array for DecimalArray {
    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn data_type(&self) -> &DataType {
        &self.data_type
    }

    fn validity(&self) -> Option<&Bitmap> {
        self.data.validity()
    }

    fn slice(&self, offset: usize, length: usize) -> Box<dyn Array> {
        Box::new(self.slice(offset, length))
    }
    unsafe fn slice_unchecked(&self, offset: usize, length: usize) -> Box<dyn Array> {
        Box::new(self.slice_unchecked(offset, length))
    }
    fn with_validity(&self, validity: Option<Bitmap>) -> Box<dyn Array> {
        Box::new(self.with_validity(validity))
    }
}

impl std::fmt::Debug for DecimalArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_fmt(self.iter(), &format!("{:?}", self.data_type()), f, false)
    }
}
