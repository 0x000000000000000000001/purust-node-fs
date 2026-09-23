// Native file descriptors. Node exposes plain numbers; the port wraps an open
// Rust file so reads and writes share one cursor per descriptor.
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

pub struct FileDescriptor {
    pub file: Mutex<std::fs::File>,
    pub append: bool,
    pub closed: AtomicBool,
}

pub fn purust_fd_new(file: std::fs::File, append: bool) -> Rc<FileDescriptor> {
    Rc::new(FileDescriptor {
        file: Mutex::new(file),
        append,
        closed: AtomicBool::new(false),
    })
}

pub fn purust_fd_closed(fd: &Rc<FileDescriptor>) -> bool {
    fd.closed.load(Ordering::Relaxed)
}

/// Shared by the sync and async modules: reads at `position` (None means the
/// current descriptor position) and mutates the target buffer.
pub fn purust_fd_read(
    fd: &Rc<FileDescriptor>,
    buffer: &Rc<Purs_Node_Buffer_Immutable::ImmutableBuffer>,
    offset: usize,
    length: usize,
    position: Option<u64>,
) -> std::io::Result<i64> {
    use std::io::{Read, Seek, SeekFrom};
    if purust_fd_closed(fd) {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "EBADF: bad file descriptor"));
    }
    let mut file = fd.file.lock().unwrap();
    if let Some(position) = position {
        file.seek(SeekFrom::Start(position))?;
    }
    let mut chunk = vec![0u8; length];
    let read = file.read(&mut chunk)?;
    buffer.with_bytes_mut(|bytes| {
        if offset < bytes.len() {
            let count = read.min(bytes.len() - offset);
            bytes[offset..offset + count].copy_from_slice(&chunk[..count]);
        }
    });
    Ok(read as i64)
}

pub fn purust_fd_write(
    fd: &Rc<FileDescriptor>,
    buffer: &Rc<Purs_Node_Buffer_Immutable::ImmutableBuffer>,
    offset: usize,
    length: usize,
    position: Option<u64>,
) -> std::io::Result<i64> {
    use std::io::{Seek, SeekFrom, Write};
    if purust_fd_closed(fd) {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "EBADF: bad file descriptor"));
    }
    let bytes = buffer.bytes();
    let available = bytes.len().saturating_sub(offset);
    let count = length.min(available);
    let mut file = fd.file.lock().unwrap();
    if let Some(position) = position {
        file.seek(SeekFrom::Start(position))?;
    } else if fd.append {
        file.seek(SeekFrom::End(0))?;
    }
    file.write_all(&bytes[offset..offset + count])?;
    Ok(count as i64)
}

pub fn purust_fd_close(fd: &Rc<FileDescriptor>) -> std::io::Result<()> {
    use std::io::Write;
    if !purust_fd_closed(fd) {
        let mut file = fd.file.lock().unwrap();
        file.flush()?;
        fd.closed.store(true, Ordering::Relaxed);
    }
    Ok(())
}

pub fn purust_fd_box(fd: Rc<FileDescriptor>) -> crate::UnknownType {
    crate::Value::Class(Rc::new(fd))
}

pub fn purust_fd_unbox(value: &crate::UnknownType) -> Rc<FileDescriptor> {
    value.unwrap_class::<Rc<FileDescriptor>>().clone()
}

/// Shared open flags/options mapping (Node's `open` flag strings).
pub fn purust_open_options(flags: &str, mode: u32) -> std::fs::OpenOptions {
    use std::fs::OpenOptions;
    let mut options = OpenOptions::new();
    match flags.trim() {
        "r" | "rs" | "sr" => {
            options.read(true);
        }
        "r+" | "rs+" | "sr+" => {
            options.read(true).write(true);
        }
        "w" => {
            options.write(true).create(true).truncate(true);
        }
        "wx" | "xw" => {
            options.write(true).create_new(true);
        }
        "w+" => {
            options.read(true).write(true).create(true).truncate(true);
        }
        "wx+" | "xw+" => {
            options.read(true).write(true).create_new(true);
        }
        "a" => {
            options.append(true).create(true);
        }
        "ax" | "xa" => {
            options.append(true).create_new(true);
        }
        "a+" => {
            options.read(true).append(true).create(true);
        }
        "ax+" | "xa+" => {
            options.read(true).append(true).create_new(true);
        }
        other => panic!("Node.FS: unsupported file flags '{other}'"),
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    let _ = mode;
    options
}

/// `Nullable FilePosition` -> `Option<u64>`.
pub fn purust_nullable_position(value: &crate::UnknownType) -> Option<u64> {
    let nullable = value.unwrap_class::<Rc<Purs_Data_Nullable::Nullable>>();
    nullable
        .value()
        .map(|value| value.unwrap_int().max(0) as u64)
}

static MKDTEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Deterministic-but-unique temp name so test output stays stable across runs
/// while repeated calls do not collide.
pub fn purust_mkdtemp(prefix: &str, encoding: &str) -> std::io::Result<String> {
    for _ in 0..1000 {
        let index = MKDTEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut value = index.wrapping_mul(2_654_435_761).wrapping_add(12_345);
        let mut suffix = String::with_capacity(6);
        for _ in 0..6 {
            suffix.push(char::from_digit((value % 36) as u32, 36).unwrap_or('0'));
            value /= 36;
        }
        let candidate = format!("{prefix}{suffix}");
        match std::fs::create_dir(&candidate) {
            Ok(_) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "EEXIST",
    ))
}
