//! Mount an existing MEM8 store read-only at an explicitly supplied mountpoint.

use anyhow::{bail, Result};
use mem8_fs_lite::{mount::Mem8FuseFs, Mem8Fs};
use std::{env, sync::Arc};

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 2 {
        bail!("usage: filesystem <storage-directory> <mountpoint>");
    }
    let storage = Arc::new(Mem8Fs::new(&args[0])?);
    Mem8FuseFs::new(storage).mount(&args[1])
}
