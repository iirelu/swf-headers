use std::ops::Range;

/// Helper trait for getting a range of bits from a container of bytes as a u32.
///
/// This is useful as the SWF spec employs the use of what it calls bit fields
/// a lot, which are non-byte-aligned numbers and data. It's publicly
/// re-exported so that if you want to continue parsing the SWF, you don't
/// have to reimplement it yourself.
pub trait BitRange {
    /// Takes a range and converts the bits in that range into a u32.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf_headers::BitRange;
    /// let vec: Vec<u8> = vec![0b0010_1100, 0b0111_0010];
    /// assert!(vec.get_bit_range(2..12) == 0b1011000111);
    /// ```
    fn get_bit_range(&self, range: Range<u32>) -> u32;
}

impl BitRange for Vec<u8> {
    fn get_bit_range(&self, range: Range<u32>) -> u32 {
        let start_bit = range.start;
        let end_bit = range.end;
        let length = end_bit - start_bit;

        assert!(end_bit/8 <= self.len() as u32);
        assert!(length < 32);

        let mut result: u32 = 0;
        for (i, off) in (start_bit..end_bit).zip(1..) {
            result |= get_x_bit(self, i) << (length-off);
        }
        result
    }
}

fn get_x_bit(bytes: &Vec<u8>, bit: u32) -> u32 {
    assert!(bit/8 < bytes.len() as u32);

    let byte = bytes[(bit/8) as usize] as u32;
    (byte >> 7-bit%8) & 1
}
