use std::rc::Rc;

fn purust_fs_error(operation: &str, path: &str, error: std::io::Error) -> crate::UnknownType {
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
    Purs_Effect_Exception::Effect_Exception_error(purust_core::purust_string_from_utf8(
        &format!("{code}: {error}, {operation} '{path}'"),
    ))
}

fn purust_fs_dispatch<W>(operation: &'static str, path: String, callback: crate::UnknownType, work: W)
where
    W: FnOnce() -> std::io::Result<crate::UnknownType> + Send,
    W: 'static,
{
    // Native IO remains active until its queued callback completes. Blocking
    // filesystem operations never run on an Aff callback or scheduler worker.
    Purs_Effect_Aff::purust_aff_spawn_native(async move {
        match tokio::task::spawn_blocking(work).await {
            Ok(result) => result.map_err(|error| purust_fs_error(operation, &path, error)),
            Err(error) if error.is_panic() => std::panic::resume_unwind(error.into_panic()),
            Err(error) => panic!("Node.FS.Async worker unexpectedly cancelled: {error}"),
        }
    }, move |result| {
        let (error, value) = match result {
            Ok(value) => (Purs_Data_Nullable::Data_Nullable_null(), value),
            Err(error) => (Purs_Data_Nullable::Data_Nullable_notNull(error), crate::Value::Unit),
        };
        // Callback's first argument is the native Nullable Error carrier,
        // not a JavaScript-style null/Error value.
        callback.unwrap_func2()(crate::Value::Class(Rc::new(error)), value);
    });
}

fn purust_fs_path(path: crate::UnknownType) -> String {
    purust_core::purust_string_to_utf8_lossy(&path.unwrap_string())
}

fn purust_fs_require_utf8(options: &crate::UnknownType) {
    let fields = options.__purust_foreign_object();
    let encoding = fields.get("encoding")
        .expect("Node.FS.Async: only qualified text-file options are implemented")
        .unwrap_string().to_ascii_lowercase();
    assert!(encoding == "utf8" || encoding == "utf-8",
        "Node.FS.Async: only qualified UTF-8 text IO is implemented");
    // Fail closed on additional options rather than silently ignoring flags.
    assert!(fields.entries().iter().all(|(key, _)| key == "encoding"),
        "Node.FS.Async: additional text-file options are not qualified");
}

pub fn Node_FS_Async_mkdirImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(|path, options, callback| {
        let path = purust_fs_path(path);
        let fields = options.__purust_foreign_object();
        let recursive = fields.get("recursive").expect("mkdir recursive option").unwrap_bool();
        let mode = fields.get("mode").expect("mkdir mode option").unwrap_string();
        let mode = u32::from_str_radix(&mode, 8).expect("mkdir octal mode");
        purust_fs_dispatch("mkdir", path.clone(), callback, move || {
            let mut builder = std::fs::DirBuilder::new();
            builder.recursive(recursive);
            #[cfg(unix)] {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(mode);
            }
            builder.create(&path)?;
            Ok(crate::Value::Unit)
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_readFileImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(|path, options, callback| {
        let path = purust_fs_path(path);
        purust_fs_require_utf8(&options);
        purust_fs_dispatch("open", path.clone(), callback, move || {
            let bytes = std::fs::read(&path)?;
            // Node's UTF-8 decoder replaces malformed byte sequences.
            Ok(crate::Value::String(purust_core::purust_string_from_utf8(&String::from_utf8_lossy(&bytes))))
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_writeFileImpl() -> crate::UnknownType {
    crate::Value::Func4(purust_core::Func4::Shared(Rc::new(|path, content, options, callback| {
        let path = purust_fs_path(path);
        purust_fs_require_utf8(&options);
        let bytes = purust_core::purust_string_to_utf8_lossy(&content.unwrap_string()).into_bytes();
        purust_fs_dispatch("open", path.clone(), callback, move || {
            std::fs::write(&path, bytes)?;
            Ok(crate::Value::Unit)
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_renameImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(|source, destination, callback| {
        let source = purust_fs_path(source);
        let destination = purust_fs_path(destination);
        let context = format!("{source}' -> '{destination}");
        purust_fs_dispatch("rename", context, callback, move || {
            std::fs::rename(source, destination)?;
            Ok(crate::Value::Unit)
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_unlinkImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, callback| {
        let path = purust_fs_path(path);
        purust_fs_dispatch("unlink", path.clone(), callback, move || {
            std::fs::remove_file(path)?;
            Ok(crate::Value::Unit)
        });
        crate::Value::Unit
    })))
}
