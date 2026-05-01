//! Type conversion utilities

use crate::error::Result;

/// Convert to field element
pub trait ToField {
    fn to_field(&self) -> Result<String>;
}

/// Convert from field element
pub trait FromField: Sized {
    fn from_field(field: &str) -> Result<Self>;
}

impl ToField for [u8; 32] {
    fn to_field(&self) -> Result<String> {
        use num_bigint::BigInt;
        
        let mut bytes = self.to_vec();
        bytes.reverse();
        let big_int = BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes);
        Ok(big_int.to_string())
    }
}

impl ToField for u64 {
    fn to_field(&self) -> Result<String> {
        Ok(self.to_string())
    }
}

impl ToField for u32 {
    fn to_field(&self) -> Result<String> {
        Ok(self.to_string())
    }
}