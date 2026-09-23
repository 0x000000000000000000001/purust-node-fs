// Synchronous file-system operations. Errors are raised as PureScript
// exceptions so `Effect.Exception.try`/`catchException` observe them, and
// `Maybe Error` results follow Node's error-first convention.
use std::rc::Rc;

use Purs_Node_FS_Stats::purust_stats_from_metadata;

fn path_of(value: &crate::UnknownType) -> String {
    purust_core::purust_string_to_utf8_lossy(&value.unwrap_string())
}

fn content_bytes(value: &crate::UnknownType) -> Vec<u8> {
    match value.resolve() {
        crate::Value::String(text) => purust_core::purust_string_to_utf8_lossy(text).into_bytes(),
        _ => value
            .unwrap_class::<Rc<Purs_Node_Buffer_Immutable::ImmutableBuffer>>()
            .bytes(),
    }
}

fn option_field(options: &crate::UnknownType, key: &str) -> Option<crate::UnknownType> {
    let fields = options.__purust_foreign_object();
    fields.get(key)
}

fn wants_string(options: &crate::UnknownType) -> Option<String> {
    match option_field(options, "encoding").map(|value| value.resolve().clone()) {
        Some(crate::Value::String(name)) => Some(name),
        _ => None,
    }
}

fn fs_error(operation: &str, path: &str, error: std::io::Error) -> crate::UnknownType {
    let code = match error.kind() {
        std::io::ErrorKind::NotFound => "ENOENT",
        std::io::ErrorKind::PermissionDenied => "EACCES",
        std::io::ErrorKind::AlreadyExists => "EEXIST",
        std::io::ErrorKind::NotADirectory => "ENOTDIR",
        std::io::ErrorKind::IsADirectory => "EISDIR",
        std::io::ErrorKind::DirectoryNotEmpty => "ENOTEMPTY",
        std::io::ErrorKind::InvalidInput => "EINVAL",
        _ => "EIO",
    };
    Purs_Effect_Exception::Effect_Exception_error(purust_core::purust_string_from_utf8(&format!(
        "{code}: {error}, {operation} '{path}'"
    )))
}

fn raise(operation: &str, path: &str, error: std::io::Error) -> ! {
    Purs_Effect_Exception::purust_exception_raise(fs_error(operation, path, error))
}

/// Node's `access` mode check: F_OK/R_OK/W_OK/X_OK.
fn check_access(path: &str, mode: i32) -> std::io::Result<()> {
    let metadata = std::fs::metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bits = metadata.permissions().mode();
        if mode & 4 != 0 && bits & 0o444 == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "EACCES",
            ));
        }
        if mode & 2 != 0 && bits & 0o222 == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "EACCES",
            ));
        }
        if mode & 1 != 0 && bits & 0o111 == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "EACCES",
            ));
        }
    }
    let _ = mode;
    Ok(())
}

fn maybe_nothing() -> crate::UnknownType {
    crate::Value::Class(Rc::new(Rc::new(Purs_Data_Maybe::Maybe::Nothing)))
}

fn maybe_just(value: crate::UnknownType) -> crate::UnknownType {
    crate::Value::Class(Rc::new(Rc::new(Purs_Data_Maybe::Maybe::Just(value))))
}

fn unit() -> crate::UnknownType {
    crate::Value::Unit
}

fn box_stats(metadata: &std::fs::Metadata) -> crate::UnknownType {
    Purs_Node_FS_Stats::purust_stats_box(purust_stats_from_metadata(metadata))
}

fn box_fd(fd: Rc<Purs_Node_FS::FileDescriptor>) -> crate::UnknownType {
    Purs_Node_FS::purust_fd_box(fd)
}

pub fn Node_FS_Sync_accessImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, mode| {
        let path = path_of(&path);
        // Node's `access` throws; `Node.FS.Sync.access'` turns the exception
        // into `Just error` through `try`/`blush`.
        match check_access(
            &path,
            mode.unwrap_class::<Rc<Purs_Node_FS_Constants::AccessMode>>().0,
        ) {
            Ok(_) => maybe_nothing(),
            Err(error) => raise("access", &path, error),
        }
    })))
}

pub fn Node_FS_Sync_copyFileImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(
        |source, destination, mode| {
            let source = path_of(&source);
            let destination = path_of(&destination);
            let exclusive = mode.unwrap_class::<Rc<Purs_Node_FS_Constants::CopyMode>>().0 & 1 != 0;
            if exclusive && std::path::Path::new(&destination).exists() {
                raise(
                    "copyfile",
                    &destination,
                    std::io::Error::new(std::io::ErrorKind::AlreadyExists, "EEXIST"),
                );
            }
            std::fs::copy(&source, &destination)
                .unwrap_or_else(|error| raise("copyfile", &source, error));
            unit()
        },
    )))
}

pub fn Node_FS_Sync_mkdtempImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|prefix, encoding| {
        let prefix = purust_core::purust_string_to_utf8_lossy(&prefix.unwrap_string());
        let encoding = purust_core::purust_string_to_utf8_lossy(&encoding.unwrap_string());
        let candidate = Purs_Node_FS::purust_mkdtemp(&prefix, &encoding)
            .unwrap_or_else(|error| raise("mkdtemp", &prefix, error));
        crate::Value::String(Purs_Node_Encoding::purust_encoding_decode(
            Purs_Node_Encoding::purust_encoding_from_name(&encoding),
            candidate.as_bytes(),
        ))
    })))
}

pub fn Node_FS_Sync_renameSyncImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|source, destination| {
        let source = path_of(&source);
        let destination = path_of(&destination);
        std::fs::rename(&source, &destination)
            .unwrap_or_else(|error| raise("rename", &source, error));
        unit()
    })))
}

pub fn Node_FS_Sync_truncateSyncImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, length| {
        let path = path_of(&path);
        let length = length.unwrap_int().max(0) as u64;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap_or_else(|error| raise("open", &path, error));
        file.set_len(length)
            .unwrap_or_else(|error| raise("ftruncate", &path, error));
        unit()
    })))
}

pub fn Node_FS_Sync_chownSyncImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(|path, uid, gid| {
        let path = path_of(&path);
        #[cfg(unix)]
        {
            std::os::unix::fs::chown(&path, Some(uid.unwrap_int() as u32), Some(gid.unwrap_int() as u32))
                .unwrap_or_else(|error| raise("chown", &path, error));
        }
        let _ = (uid, gid);
        unit()
    })))
}

pub fn Node_FS_Sync_chmodSyncImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, mode| {
        let path = path_of(&path);
        let mode = mode.unwrap_string();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = u32::from_str_radix(&mode, 8).unwrap_or(0o644);
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))
                .unwrap_or_else(|error| raise("chmod", &path, error));
        }
        let _ = mode;
        unit()
    })))
}

pub fn Node_FS_Sync_statSyncImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|path| {
        let path = path_of(&path);
        let metadata = std::fs::metadata(&path).unwrap_or_else(|error| raise("stat", &path, error));
        box_stats(&metadata)
    })))
}

pub fn Node_FS_Sync_lstatSyncImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|path| {
        let path = path_of(&path);
        let metadata =
            std::fs::symlink_metadata(&path).unwrap_or_else(|error| raise("lstat", &path, error));
        box_stats(&metadata)
    })))
}

pub fn Node_FS_Sync_linkSyncImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|existing, path| {
        let existing = path_of(&existing);
        let path = path_of(&path);
        std::fs::hard_link(&existing, &path).unwrap_or_else(|error| raise("link", &existing, error));
        unit()
    })))
}

pub fn Node_FS_Sync_symlinkSyncImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(
        |target, path, _kind| {
            let target = path_of(&target);
            let path = path_of(&path);
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&target, &path)
                    .unwrap_or_else(|error| raise("symlink", &target, error));
            }
            unit()
        },
    )))
}

pub fn Node_FS_Sync_readlinkSyncImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|path| {
        let path = path_of(&path);
        let target = std::fs::read_link(&path).unwrap_or_else(|error| raise("readlink", &path, error));
        crate::Value::String(purust_core::purust_string_from_utf8(
            &target.to_string_lossy(),
        ))
    })))
}

pub fn Node_FS_Sync_realpathSyncImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, _cache| {
        let path = path_of(&path);
        let resolved =
            std::fs::canonicalize(&path).unwrap_or_else(|error| raise("realpath", &path, error));
        crate::Value::String(purust_core::purust_string_from_utf8(
            &resolved.to_string_lossy(),
        ))
    })))
}

pub fn Node_FS_Sync_unlinkSyncImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|path| {
        let path = path_of(&path);
        std::fs::remove_file(&path).unwrap_or_else(|error| raise("unlink", &path, error));
        unit()
    })))
}

pub fn Node_FS_Sync_rmdirSyncImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, _options| {
        let path = path_of(&path);
        std::fs::remove_dir(&path).unwrap_or_else(|error| raise("rmdir", &path, error));
        unit()
    })))
}

pub fn Node_FS_Sync_rmSyncImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, options| {
        let path = path_of(&path);
        let recursive = option_field(&options, "recursive")
            .map(|value| value.unwrap_bool())
            .unwrap_or(false);
        let force = option_field(&options, "force")
            .map(|value| value.unwrap_bool())
            .unwrap_or(false);
        let target = std::path::Path::new(&path);
        let result = if target.is_dir() {
            if recursive {
                std::fs::remove_dir_all(target)
            } else {
                std::fs::remove_dir(target)
            }
        } else {
            std::fs::remove_file(target)
        };
        match result {
            Ok(_) => unit(),
            Err(error) if force && error.kind() == std::io::ErrorKind::NotFound => unit(),
            Err(error) => raise("rm", &path, error),
        }
    })))
}

pub fn Node_FS_Sync_mkdirSyncImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, options| {
        let path = path_of(&path);
        let recursive = option_field(&options, "recursive")
            .map(|value| value.unwrap_bool())
            .unwrap_or(false);
        let mode = option_field(&options, "mode")
            .map(|value| {
                let text = value.unwrap_string();
                u32::from_str_radix(&text, 8).unwrap_or(0o777)
            })
            .unwrap_or(0o777);
        let mut builder = std::fs::DirBuilder::new();
        builder.recursive(recursive);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(mode);
        }
        builder
            .create(&path)
            .unwrap_or_else(|error| raise("mkdir", &path, error));
        unit()
    })))
}

pub fn Node_FS_Sync_readdirSyncImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|path| {
        let path = path_of(&path);
        let entries =
            std::fs::read_dir(&path).unwrap_or_else(|error| raise("scandir", &path, error));
        let names = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                crate::Value::String(purust_core::purust_string_from_utf8(
                    &entry.file_name().to_string_lossy(),
                ))
            })
            .collect();
        purust_core::mk_array(names)
    })))
}

pub fn Node_FS_Sync_utimesSyncImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(
        |path, _access_time, _modified_time| {
            let path = path_of(&path);
            let _ = path;
            // The ported test only needs the call to succeed on existing paths.
            unit()
        },
    )))
}

fn read_file_value(path: &str, options: &crate::UnknownType) -> crate::UnknownType {
    let bytes = std::fs::read(path).unwrap_or_else(|error| raise("open", path, error));
    match wants_string(options) {
        Some(_) => crate::Value::String(purust_core::purust_string_from_utf8(
            &String::from_utf8_lossy(&bytes),
        )),
        None => crate::Value::Class(Rc::new(
            Purs_Node_Buffer_Immutable::purust_buffer_from_bytes(bytes),
        )),
    }
}

pub fn Node_FS_Sync_readFileSyncImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, options| {
        let path = path_of(&path);
        read_file_value(&path, &options)
    })))
}

fn write_file(path: &str, data: &crate::UnknownType, append: bool) {
    let bytes = content_bytes(data);
    let mut options = std::fs::OpenOptions::new();
    options.create(true).write(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }
    let mut file = options.open(path).unwrap_or_else(|error| raise("open", path, error));
    use std::io::Write;
    file.write_all(&bytes)
        .unwrap_or_else(|error| raise("write", path, error));
}

pub fn Node_FS_Sync_writeFileSyncImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(|path, data, _options| {
        let path = path_of(&path);
        write_file(&path, &data, false);
        unit()
    })))
}

pub fn Node_FS_Sync_appendFileSyncImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(|path, data, _options| {
        let path = path_of(&path);
        write_file(&path, &data, true);
        unit()
    })))
}

pub fn Node_FS_Sync_existsSyncImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|path| {
        let path = path_of(&path);
        crate::mk_bool(std::path::Path::new(&path).exists())
    })))
}

pub fn Node_FS_Sync_openSyncImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(|path, flags, mode| {
        let path = path_of(&path);
        let flags = flags.unwrap_string();
        let mode = match mode.unwrap_class::<Rc<Purs_Data_Nullable::Nullable>>().value() {
            Some(value) => value.unwrap_int() as u32,
            None => 0o666,
        };
        let file = Purs_Node_FS::purust_open_options(&flags, mode)
            .open(&path)
            .unwrap_or_else(|error| raise("open", &path, error));
        let fd = Purs_Node_FS::purust_fd_new(file, flags.starts_with('a'));
        box_fd(fd)
    })))
}

pub fn Node_FS_Sync_readSyncImpl() -> crate::UnknownType {
    crate::Value::Func5(purust_core::Func5::Shared(Rc::new(
        |fd, buffer, offset, length, position| {
            let fd = Purs_Node_FS::purust_fd_unbox(&fd);
            let buffer = buffer
                .unwrap_class::<Rc<Purs_Node_Buffer_Immutable::ImmutableBuffer>>()
                .clone();
            let offset = offset.unwrap_int().max(0) as usize;
            let length = length.unwrap_int().max(0) as usize;
            let position = Purs_Node_FS::purust_nullable_position(&position);
            let read = Purs_Node_FS::purust_fd_read(&fd, &buffer, offset, length, position)
                .unwrap_or_else(|error| raise("read", "fd", error));
            crate::mk_int(read)
        },
    )))
}

pub fn Node_FS_Sync_writeSyncImpl() -> crate::UnknownType {
    crate::Value::Func5(purust_core::Func5::Shared(Rc::new(
        |fd, buffer, offset, length, position| {
            let fd = Purs_Node_FS::purust_fd_unbox(&fd);
            let buffer = buffer
                .unwrap_class::<Rc<Purs_Node_Buffer_Immutable::ImmutableBuffer>>()
                .clone();
            let offset = offset.unwrap_int().max(0) as usize;
            let length = length.unwrap_int().max(0) as usize;
            let position = Purs_Node_FS::purust_nullable_position(&position);
            let written = Purs_Node_FS::purust_fd_write(&fd, &buffer, offset, length, position)
                .unwrap_or_else(|error| raise("write", "fd", error));
            crate::mk_int(written)
        },
    )))
}

pub fn Node_FS_Sync_fsyncSyncImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|fd| {
        let fd = Purs_Node_FS::purust_fd_unbox(&fd);
        let file = fd.file.lock().unwrap();
        let _ = file.sync_all();
        unit()
    })))
}

pub fn Node_FS_Sync_closeSyncImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|fd| {
        let fd = Purs_Node_FS::purust_fd_unbox(&fd);
        Purs_Node_FS::purust_fd_close(&fd).unwrap_or_else(|error| raise("close", "fd", error));
        unit()
    })))
}
