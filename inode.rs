extern crate time;

use time::Timespec;

type Page = Box<([u8, ..4096])>;
type Entry = Option<Page>;
type TList<T> = Box<([T, ..256])>;
type EntryList = TList<Entry>;
type DoubleEntryList = TList<Box<EntryList>>;

pub struct Inode {
  single: EntryList,
  data2: DoubleEntryList,
  size: uint,

  mod_time: Timespec,
  access_time: Timespec,
  create_time: Timespec,
}
