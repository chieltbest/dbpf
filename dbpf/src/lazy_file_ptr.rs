use std::fmt::{Debug, Display, Formatter};
use std::io::{Read, Seek, SeekFrom};
use binrw::{BinRead, BinResult, binrw, Endian, NamedArgs};
use binrw::file_ptr::IntoSeekFrom;

#[derive(Clone)]
pub struct LazyFilePtr<Ptr, T: BinRead, Args: Clone> {
    pub ptr: Ptr,
    pub endian: Endian,
    pub args: LazyFilePtrArgs<Args>,
    data: Option<T>,
}

impl<Ptr, T, Args> BinRead for LazyFilePtr<Ptr, T, Args>
    where for<'a> Ptr: BinRead<Args<'a>=()> + IntoSeekFrom,
          T: BinRead,
          for<'a> T::Args<'a>: Clone + 'a,
          Args: Clone,
{
    type Args<'b> = LazyFilePtrArgs<Args>;

    fn read_options<R: Read + Seek>(reader: &mut R, options: Endian, args: Self::Args<'_>) -> BinResult<Self> {
        Ok(Self {
            ptr: Ptr::read_options(reader, options, ())?,
            endian: options.clone(),
            args,
            data: None,
        })
    }
}

impl<'a, Ptr: IntoSeekFrom, T: BinRead> LazyFilePtr<Ptr, T, T::Args<'a>> where T::Args<'a>: Clone {
    pub fn get<R: Read + Seek>(&mut self, reader: &mut R) -> BinResult<&mut T> {
        if let Some(ref mut data) = self.data {
            Ok(data)
        } else {
            let relative_to = self.args.offset;
            let before = reader.seek(SeekFrom::Start(relative_to))?;
            reader.seek(self.ptr.into_seek_from())?;

            let mut inner = T::read_options(reader, self.endian, self.args.inner.clone())?;
            inner.after_parse(reader, self.endian, self.args.inner.clone())?;

            reader.seek(SeekFrom::Start(before))?;

            Ok(self.data.insert(
                inner
            ))
        }
    }
}

impl<'a, Ptr: IntoSeekFrom, T: BinRead + Debug> Debug for LazyFilePtr<Ptr, T, T::Args<'a>> where T::Args<'a>: Clone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyFilePtr")
            .field("ptr", &self.ptr)
            .field("endian", &self.endian)
            .field("offset", &self.args.offset)
            // .field("args", &self.args)
            .field("data", &self.data)
            .finish()
    }
}

/// Named arguments for the [`BinRead::read_options()`] implementation of [`LazyFilePtr`].
///
/// The `inner` field can be omitted completely if the inner type doesn’t
/// require arguments, in which case a default value will be used.
#[derive(Clone, Default, Debug, NamedArgs)]
pub struct LazyFilePtrArgs<Inner: Clone> {
    /// An absolute offset added to the [`LazyFilePtr::ptr`](crate::LazyFilePtr::ptr)
    /// offset before reading the pointed-to value.
    #[named_args(default = 0)]
    pub offset: u64,

    /// The [arguments](crate::BinRead::Args) for the inner type.
    // #[named_args(try_optional)]
    pub inner: Inner,
}

#[binrw]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct Zero {}

impl IntoSeekFrom for Zero {
    fn into_seek_from(self) -> SeekFrom {
        SeekFrom::Current(0)
    }
}

impl Display for Zero {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&0, f)
    }
}
