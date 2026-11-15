use super::Error;

pub(super) enum FieldValue<'a> {
    Integer(u64),
    Bytes(&'a [u8]),
    Fixed,
}

pub(super) trait Read: Sized {
    fn read(value: FieldValue<'_>, remaining: &mut usize) -> Result<Self, Error>;
}

impl Read for u64 {
    fn read(value: FieldValue<'_>, _: &mut usize) -> Result<Self, Error> {
        match value {
            FieldValue::Integer(value) => Ok(value),
            _ => Err(Error::Malformed),
        }
    }
}
impl Read for bool {
    fn read(value: FieldValue<'_>, remaining: &mut usize) -> Result<Self, Error> {
        match u64::read(value, remaining)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Malformed),
        }
    }
}
impl<T: Read> Read for Option<T> {
    fn read(value: FieldValue<'_>, remaining: &mut usize) -> Result<Self, Error> {
        T::read(value, remaining).map(Some)
    }
}

pub(super) struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], Error> {
        let (head, tail) = self.rest.split_at_checked(count).ok_or(Error::Malformed)?;
        self.rest = tail;
        Ok(head)
    }

    fn integer(&mut self) -> Result<u64, Error> {
        let mut value = 0u64;
        for shift in (0..70).step_by(7) {
            let byte = self.take(1)?[0];
            if shift == 63 && byte > 1 {
                return Err(Error::Malformed);
            }
            value |= u64::from(byte & 127) << shift;
            if byte & 128 == 0 {
                return Ok(value);
            }
        }
        Err(Error::Malformed)
    }

    pub(super) fn next(
        &mut self,
        remaining: &mut usize,
    ) -> Result<Option<(u32, FieldValue<'a>)>, Error> {
        if self.rest.is_empty() {
            return Ok(None);
        }
        *remaining = remaining.checked_sub(1).ok_or(Error::FieldLimit)?;
        let key = self.integer()?;
        let number = key >> 3;
        if number == 0 || number > 0x1fff_ffff {
            return Err(Error::Malformed);
        }
        let value = match key & 7 {
            0 => FieldValue::Integer(self.integer()?),
            1 => {
                self.take(8)?;
                FieldValue::Fixed
            }
            2 => {
                let count = usize::try_from(self.integer()?).map_err(|_| Error::Malformed)?;
                FieldValue::Bytes(self.take(count)?)
            }
            5 => {
                self.take(4)?;
                FieldValue::Fixed
            }
            _ => return Err(Error::UnsupportedWireType),
        };
        Ok(Some((number as u32, value)))
    }
}
