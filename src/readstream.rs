use std::mem::size_of;
use std::ops::BitOrAssign;

use num_traits::{Float, PrimInt};

use crate::endianness::Endianness;
use crate::num_traits::{IsSigned, UncheckedPrimitiveFloat, UncheckedPrimitiveInt};
use crate::BitReadBuffer;
use crate::{BitError, BitRead, BitReadSized, Result};
use std::cmp::min;

/// Stream that provides an easy way to iterate trough a [`BitBuffer`]
///
/// # Examples
///
/// ```
/// use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian};
///
/// let bytes = vec![
///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
/// ];
/// let buffer = BitReadBuffer::new(bytes, LittleEndian);
/// let mut stream = BitReadStream::new(buffer);
/// ```
///
/// [`BitBuffer`]: struct.BitBuffer.html
#[derive(Debug)]
pub struct BitReadStream<E>
where
    E: Endianness,
{
    buffer: BitReadBuffer<E>,
    start_pos: usize,
    pos: usize,
}

impl<E> BitReadStream<E>
where
    E: Endianness,
{
    /// Create a new stream from a [`BitBuffer`]
    ///
    /// # Examples
    ///
    /// ```
    /// use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian};
    ///
    /// let bytes = vec![
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitReadStream::new(buffer);
    /// ```
    ///
    /// [`BitBuffer`]: struct.BitBuffer.html
    pub fn new(buffer: BitReadBuffer<E>) -> Self {
        BitReadStream {
            start_pos: 0,
            pos: 0,
            buffer,
        }
    }

    /// Read a single bit from the stream as boolean
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.read_bool()?, true);
    /// assert_eq!(stream.read_bool()?, false);
    /// assert_eq!(stream.pos(), 2);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    #[inline]
    pub fn read_bool(&mut self) -> Result<bool> {
        let result = self.buffer.read_bool(self.pos);
        if result.is_ok() {
            self.pos += 1;
        }
        result
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_bool_unchecked(&mut self) -> bool {
        let result = self.buffer.read_bool_unchecked(self.pos);
        self.pos += 1;
        result
    }

    /// Read a sequence of bits from the stream as integer
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    /// - [`ReadError::TooManyBits`]: to many bits requested for the chosen integer type
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.read_int::<u16>(3)?, 0b101);
    /// assert_eq!(stream.read_int::<u16>(3)?, 0b110);
    /// assert_eq!(stream.pos(), 6);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    /// [`ReadError::TooManyBits`]: enum.ReadError.html#variant.TooManyBits
    #[inline]
    pub fn read_int<T>(&mut self, count: usize) -> Result<T>
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt,
    {
        let result = self.buffer.read_int(self.pos, count);
        if result.is_ok() {
            self.pos += count;
        }
        result
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_int_unchecked<T>(&mut self, count: usize) -> T
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt,
    {
        let result = self.buffer.read_int_unchecked(self.pos, count);
        self.pos += count;
        result
    }

    /// Read a sequence of bits from the stream as float
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let result = stream.read_float::<f32>()?;
    /// assert_eq!(stream.pos(), 32);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    #[inline]
    pub fn read_float<T>(&mut self) -> Result<T>
    where
        T: Float + UncheckedPrimitiveFloat,
    {
        let count = size_of::<T>() * 8;
        let result = self.buffer.read_float(self.pos);
        if result.is_ok() {
            self.pos += count;
        }
        result
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_float_unchecked<T>(&mut self) -> T
    where
        T: Float + UncheckedPrimitiveFloat,
    {
        let count = size_of::<T>() * 8;
        let result = self.buffer.read_float_unchecked(self.pos);
        self.pos += count;
        result
    }

    /// Read a series of bytes from the stream
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.read_bytes(3)?, &[0b1011_0101, 0b0110_1010, 0b1010_1100]);
    /// assert_eq!(stream.pos(), 24);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    #[inline]
    pub fn read_bytes(&mut self, byte_count: usize) -> Result<Vec<u8>> {
        let count = byte_count * 8;
        let result = self.buffer.read_bytes(self.pos, byte_count);
        if result.is_ok() {
            self.pos += count;
        }
        result
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_bytes_unchecked(&mut self, byte_count: usize) -> Vec<u8> {
        let count = byte_count * 8;
        let result = self.buffer.read_bytes_unchecked(self.pos, byte_count);
        self.pos += count;
        result
    }

    /// Read a series of bytes from the stream as utf8 string
    ///
    /// You can either read a fixed number of bytes, or a dynamic length null-terminated string
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    /// - [`ReadError::Utf8Error`]: the read bytes are not valid utf8
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0x48, 0x65, 0x6c, 0x6c,
    /// #     0x6f, 0x20, 0x77, 0x6f,
    /// #     0x72, 0x6c, 0x64, 0,
    /// #     0,    0,    0,    0
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// // Fixed length string
    /// stream.set_pos(0);
    /// assert_eq!(stream.read_string(Some(11))?, "Hello world".to_owned());
    /// assert_eq!(11 * 8, stream.pos());
    /// // fixed length with null padding
    /// stream.set_pos(0);
    /// assert_eq!(stream.read_string(Some(16))?, "Hello world".to_owned());
    /// assert_eq!(16 * 8, stream.pos());
    /// // null terminated
    /// stream.set_pos(0);
    /// assert_eq!(stream.read_string(None)?, "Hello world".to_owned());
    /// assert_eq!(12 * 8, stream.pos()); // 1 more for the terminating null byte
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    /// [`ReadError::Utf8Error`]: enum.ReadError.html#variant.Utf8Error
    #[inline]
    pub fn read_string(&mut self, byte_len: Option<usize>) -> Result<String> {
        let max_length = self.bits_left() / 8;

        let result = self.buffer.read_string(self.pos, byte_len).map_err(|err| {
            // still advance the stream on malformed utf8
            if let BitError::Utf8Error(err) = &err {
                self.pos += match byte_len {
                    Some(len) => len * 8,
                    None => min((err.as_bytes().len() + 1) * 8, max_length),
                };
            }
            err
        })?;
        let read = match byte_len {
            Some(len) => len * 8,
            None => (result.len() + 1) * 8,
        };

        // due to how sub buffer/streams work, the result string can be longer than the current stream
        // (but not the top level buffer)
        // thus we trim the resulting string to make sure it fits in the source stream
        if read > self.bits_left() {
            // find the maximum well-formed utf8 string that fits in max_len
            let mut acc = String::new();
            for c in result.chars() {
                if acc.len() + c.len_utf8() > max_length {
                    break;
                }
                acc.push(c);
            }
            self.pos += acc.len() * 8;
            return Ok(acc);
        }
        self.pos += read;
        Ok(result)
    }

    /// Read a sequence of bits from the stream as a BitStream
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let mut bits = stream.read_bits(3)?;
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(bits.pos(), 0);
    /// assert_eq!(bits.bit_len(), 3);
    /// assert_eq!(stream.read_int::<u8>(3)?, 0b110);
    /// assert_eq!(bits.read_int::<u8>(3)?, 0b101);
    /// assert_eq!(true, bits.read_int::<u8>(1).is_err());
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    pub fn read_bits(&mut self, count: usize) -> Result<Self> {
        let result = BitReadStream {
            buffer: self.buffer.get_sub_buffer(self.pos + count)?,
            start_pos: self.pos,
            pos: self.pos,
        };
        self.pos += count;
        Ok(result)
    }

    /// Skip a number of bits in the stream
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream to skip
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// stream.skip_bits(3)?;
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(stream.read_int::<u8>(3)?, 0b110);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    pub fn skip_bits(&mut self, count: usize) -> Result<()> {
        if count <= self.bits_left() {
            self.pos += count;
            Ok(())
        } else {
            Err(BitError::NotEnoughData {
                requested: count,
                bits_left: self.bits_left(),
            })
        }
    }

    /// Align the stream on the next byte and returns the amount of bits read
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream to skip
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// stream.align();
    /// assert_eq!(stream.pos(), 0);
    ///
    /// stream.skip_bits(3)?;
    /// assert_eq!(stream.pos(), 3);
    /// stream.align();
    /// assert_eq!(stream.pos(), 8);
    /// assert_eq!(stream.read_int::<u8>(4)?, 0b1010);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    pub fn align(&mut self) -> Result<usize> {
        match self.pos % 8 {
            0 => Ok(0),
            n => self.skip_bits(8 - n).map(|_| 8 - n),
        }
    }

    /// Set the position of the stream
    ///
    /// # Errors
    ///
    /// - [`ReadError::IndexOutOfBounds`]: new position is outside the bounds of the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// stream.set_pos(3)?;
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(stream.read_int::<u8>(3)?, 0b110);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::IndexOutOfBounds`]: enum.ReadError.html#variant.IndexOutOfBounds
    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        if pos > self.bit_len() {
            return Err(BitError::IndexOutOfBounds {
                pos,
                size: self.bit_len(),
            });
        }
        self.pos = pos + self.start_pos;
        Ok(())
    }

    /// Get the length of the stream in bits
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.bit_len(), 64);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn bit_len(&self) -> usize {
        self.buffer.bit_len() - self.start_pos
    }

    /// Get the current position in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.pos(), 0);
    /// stream.skip_bits(5)?;
    /// assert_eq!(stream.pos(), 5);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn pos(&self) -> usize {
        self.pos - self.start_pos
    }

    /// Get the number of bits left in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.bits_left(), 64);
    /// stream.skip_bits(5)?;
    /// assert_eq!(stream.bits_left(), 59);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn bits_left(&self) -> usize {
        self.bit_len() - self.pos()
    }

    /// Read a value based on the provided type
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let int: u8 = stream.read()?;
    /// assert_eq!(int, 0b1011_0101);
    /// let boolean: bool = stream.read()?;
    /// assert_eq!(false, boolean);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// use bitbuffer::BitRead;
    /// #
    /// #[derive(BitRead, Debug, PartialEq)]
    /// struct ComplexType {
    ///     first: u8,
    ///     #[size = 15]
    ///     second: u16,
    ///     third: bool,
    /// }
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let data: ComplexType = stream.read()?;
    /// assert_eq!(data, ComplexType {
    ///     first: 0b1011_0101,
    ///     second: 0b010_1100_0110_1010,
    ///     third: true,
    /// });
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn read<T: BitRead<E>>(&mut self) -> Result<T> {
        T::read(self)
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_unchecked<T: BitRead<E>>(&mut self) -> Result<T> {
        T::read_unchecked(self)
    }

    /// Read a value based on the provided type and size
    ///
    /// The meaning of the size parameter differs depending on the type that is being read
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let int: u8 = stream.read_sized(7)?;
    /// assert_eq!(int, 0b011_0101);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let data: Vec<u16> = stream.read_sized(3)?;
    /// assert_eq!(data, vec![0b0110_1010_1011_0101, 0b1001_1001_1010_1100, 0b1001_1001_1001_1001]);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn read_sized<T: BitReadSized<E>>(&mut self, size: usize) -> Result<T> {
        T::read(self, size)
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_sized_unchecked<T: BitReadSized<E>>(&mut self, size: usize) -> Result<T> {
        T::read_unchecked(self, size)
    }

    /// Check if we can read a number of bits from the stream
    pub fn check_read(&self, count: usize) -> Result<()> {
        if self.bits_left() < count {
            Err(BitError::NotEnoughData {
                requested: count,
                bits_left: self.bits_left(),
            })
        } else {
            Ok(())
        }
    }
}

impl<E: Endianness> Clone for BitReadStream<E> {
    fn clone(&self) -> Self {
        BitReadStream {
            buffer: self.buffer.clone(),
            start_pos: self.pos,
            pos: self.pos,
        }
    }
}

impl<E: Endianness> From<BitReadBuffer<E>> for BitReadStream<E> {
    fn from(buffer: BitReadBuffer<E>) -> Self {
        BitReadStream::new(buffer)
    }
}

impl<E: Endianness> From<Vec<u8>> for BitReadStream<E> {
    fn from(bytes: Vec<u8>) -> Self {
        BitReadStream::new(BitReadBuffer::from(bytes))
    }
}
