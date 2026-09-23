// Node-compatible `fs.Stats`. Times are kept in milliseconds so the accessors
// can hand out either JSDate values or numbers, like Node's `Stats`.
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatsKind {
    File,
    Directory,
    Symlink,
    Fifo,
    Socket,
    CharacterDevice,
    BlockDevice,
    Unknown,
}

pub struct Stats {
    pub dev: f64,
    pub ino: f64,
    pub mode: f64,
    pub nlink: f64,
    pub uid: f64,
    pub gid: f64,
    pub rdev: f64,
    pub size: f64,
    pub blksize: f64,
    pub blocks: f64,
    pub atime_ms: f64,
    pub mtime_ms: f64,
    pub ctime_ms: f64,
    pub birthtime_ms: f64,
    pub kind: StatsKind,
}

fn kind_of(file_type: &std::fs::FileType) -> StatsKind {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if file_type.is_dir() {
            StatsKind::Directory
        } else if file_type.is_file() {
            StatsKind::File
        } else if file_type.is_symlink() {
            StatsKind::Symlink
        } else if file_type.is_fifo() {
            StatsKind::Fifo
        } else if file_type.is_socket() {
            StatsKind::Socket
        } else if file_type.is_char_device() {
            StatsKind::CharacterDevice
        } else if file_type.is_block_device() {
            StatsKind::BlockDevice
        } else {
            StatsKind::Unknown
        }
    }
    #[cfg(not(unix))]
    {
        if file_type.is_dir() {
            StatsKind::Directory
        } else if file_type.is_file() {
            StatsKind::File
        } else if file_type.is_symlink() {
            StatsKind::Symlink
        } else {
            StatsKind::Unknown
        }
    }
}

fn system_time_ms(time: std::io::Result<std::time::SystemTime>) -> f64 {
    match time {
        Ok(time) => match time.duration_since(std::time::UNIX_EPOCH) {
            Ok(elapsed) => elapsed.as_millis() as f64,
            Err(error) => -(error.duration().as_millis() as f64),
        },
        Err(_) => 0.0,
    }
}

pub fn purust_stats_from_metadata(metadata: &std::fs::Metadata) -> Rc<Stats> {
    #[cfg(unix)]
    let (dev, ino, mode, nlink, uid, gid, rdev, blksize, blocks) = {
        use std::os::unix::fs::MetadataExt;
        (
            metadata.dev() as f64,
            metadata.ino() as f64,
            metadata.mode() as f64,
            metadata.nlink() as f64,
            metadata.uid() as f64,
            metadata.gid() as f64,
            metadata.rdev() as f64,
            metadata.blksize() as f64,
            metadata.blocks() as f64,
        )
    };
    #[cfg(not(unix))]
    let (dev, ino, mode, nlink, uid, gid, rdev, blksize, blocks) =
        (0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 4096.0, 0.0);
    Rc::new(Stats {
        dev,
        ino,
        mode,
        nlink,
        uid,
        gid,
        rdev,
        size: metadata.len() as f64,
        blksize,
        blocks,
        atime_ms: system_time_ms(metadata.accessed()),
        mtime_ms: system_time_ms(metadata.modified()),
        ctime_ms: metadata_ctime_ms(metadata),
        birthtime_ms: system_time_ms(metadata.created()),
        kind: kind_of(&metadata.file_type()),
    })
}

#[cfg(unix)]
fn metadata_ctime_ms(metadata: &std::fs::Metadata) -> f64 {
    use std::os::unix::fs::MetadataExt;
    metadata.ctime() as f64 * 1000.0 + metadata.ctime_nsec() as f64 / 1_000_000.0
}

#[cfg(not(unix))]
fn metadata_ctime_ms(metadata: &std::fs::Metadata) -> f64 {
    system_time_ms(metadata.modified())
}

pub fn purust_stats_box(stats: Rc<Stats>) -> crate::UnknownType {
    crate::Value::Class(Rc::new(stats))
}

fn stats_unbox(value: &crate::UnknownType) -> Rc<Stats> {
    value.unwrap_class::<Rc<Stats>>().clone()
}

fn stats_fn(project: impl Fn(&Stats) -> crate::UnknownType + 'static) -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(move |value| {
        project(&stats_unbox(&value))
    })))
}

fn is_kind(value: &crate::UnknownType, kind: StatsKind) -> crate::UnknownType {
    crate::mk_bool(stats_unbox(value).kind == kind)
}

pub fn Node_FS_Stats_isBlockDeviceImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_bool(stats.kind == StatsKind::BlockDevice))
}

pub fn Node_FS_Stats_isCharacterDeviceImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_bool(stats.kind == StatsKind::CharacterDevice))
}

pub fn Node_FS_Stats_isDirectoryImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_bool(stats.kind == StatsKind::Directory))
}

pub fn Node_FS_Stats_isFIFOImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_bool(stats.kind == StatsKind::Fifo))
}

pub fn Node_FS_Stats_isFileImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_bool(stats.kind == StatsKind::File))
}

pub fn Node_FS_Stats_isSocketImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_bool(stats.kind == StatsKind::Socket))
}

pub fn Node_FS_Stats_isSymbolicLinkImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_bool(stats.kind == StatsKind::Symlink))
}

pub fn Node_FS_Stats_devImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.dev))
}

pub fn Node_FS_Stats_inodeImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.ino))
}

pub fn Node_FS_Stats_modeImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.mode))
}

pub fn Node_FS_Stats_nlinkImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.nlink))
}

pub fn Node_FS_Stats_uidImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.uid))
}

pub fn Node_FS_Stats_gidImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.gid))
}

pub fn Node_FS_Stats_rdevImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.rdev))
}

pub fn Node_FS_Stats_sizeImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.size))
}

pub fn Node_FS_Stats_blkSizeImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.blksize))
}

pub fn Node_FS_Stats_blocksImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.blocks))
}

pub fn Node_FS_Stats_accessedTimeMsImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.atime_ms))
}

pub fn Node_FS_Stats_modifiedTimeMsImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.mtime_ms))
}

pub fn Node_FS_Stats_statusChangedTimeMsImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.ctime_ms))
}

pub fn Node_FS_Stats_birthtimeMsImpl() -> crate::UnknownType {
    stats_fn(|stats| crate::mk_number(stats.birthtime_ms))
}

fn date_value(milliseconds: f64) -> crate::UnknownType {
    crate::Value::Class(Rc::new(Purs_Data_JSDate::Data_JSDate_fromTime(milliseconds)))
}

pub fn Node_FS_Stats_accessedTimeImpl() -> crate::UnknownType {
    stats_fn(|stats| date_value(stats.atime_ms))
}

pub fn Node_FS_Stats_modifiedTimeImpl() -> crate::UnknownType {
    stats_fn(|stats| date_value(stats.mtime_ms))
}

pub fn Node_FS_Stats_statusChangedTimeImpl() -> crate::UnknownType {
    stats_fn(|stats| date_value(stats.ctime_ms))
}

pub fn Node_FS_Stats_birthTimeImpl() -> crate::UnknownType {
    stats_fn(|stats| date_value(stats.birthtime_ms))
}

pub fn Node_FS_Stats_showStatsObj(value: Rc<Stats>) -> String {
    let stats = value;
    let kind = match stats.kind {
        StatsKind::File => "file",
        StatsKind::Directory => "directory",
        StatsKind::Symlink => "symlink",
        StatsKind::Fifo => "fifo",
        StatsKind::Socket => "socket",
        StatsKind::CharacterDevice => "character device",
        StatsKind::BlockDevice => "block device",
        StatsKind::Unknown => "unknown",
    };
    format!(
        "Stats {{ dev: {}, mode: {}, nlink: {}, uid: {}, gid: {}, rdev: {}, size: {}, blksize: {}, blocks: {}, atimeMs: {}, mtimeMs: {}, ctimeMs: {}, birthtimeMs: {}, kind: {} }}",
        stats.dev, stats.mode, stats.nlink, stats.uid, stats.gid, stats.rdev, stats.size,
        stats.blksize, stats.blocks, stats.atime_ms, stats.mtime_ms, stats.ctime_ms,
        stats.birthtime_ms, kind
    )
}

// Unused directly, kept so the predicate helpers stay obviously symmetric.
#[allow(dead_code)]
fn is_kind_helper(value: &crate::UnknownType, kind: StatsKind) -> crate::UnknownType {
    is_kind(value, kind)
}
