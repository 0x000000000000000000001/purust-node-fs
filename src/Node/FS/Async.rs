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

/// Text options select a string result; without `encoding` Node returns a Buffer.
fn purust_fs_text_encoding(options: &crate::UnknownType) -> Option<String> {
    match options.__purust_foreign_object().get("encoding") {
        Some(encoding) => Some(encoding.unwrap_string()),
        None => None,
    }
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
        let encoding = purust_fs_text_encoding(&options);
        purust_fs_dispatch("open", path.clone(), callback, move || {
            let bytes = std::fs::read(&path)?;
            match encoding {
                // Node's UTF-8 decoder replaces malformed byte sequences.
                Some(_) => Ok(crate::Value::String(
                    purust_core::purust_string_from_utf8(&String::from_utf8_lossy(&bytes)),
                )),
                None => Ok(crate::Value::Class(std::sync::Arc::new(
                    Purs_Node_Buffer_Immutable::purust_buffer_from_bytes(bytes),
                ))),
            }
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_writeFileImpl() -> crate::UnknownType {
    crate::Value::Func4(purust_core::Func4::Shared(Rc::new(|path, content, options, callback| {
        let path = purust_fs_path(path);
        let _ = purust_fs_text_encoding(&options);
        let bytes = match content.resolve() {
            crate::Value::String(text) => purust_core::purust_string_to_utf8_lossy(text).into_bytes(),
            _ => content
                .unwrap_class::<std::sync::Arc<Purs_Node_Buffer_Immutable::ImmutableBuffer>>()
                .bytes(),
        };
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

fn purust_fs_dispatch_error_only<W>(
    operation: &'static str,
    path: String,
    callback: crate::UnknownType,
    work: W,
) where
    W: FnOnce() -> std::io::Result<()> + Send + 'static,
{
    Purs_Effect_Aff::purust_aff_spawn_native(
        async move {
            match tokio::task::spawn_blocking(work).await {
                Ok(Ok(())) => Ok(crate::Value::Unit),
                Ok(Err(error)) => Err(purust_fs_error(operation, &path, error)),
                Err(error) if error.is_panic() => std::panic::resume_unwind(error.into_panic()),
                Err(error) => panic!("Node.FS.Async worker unexpectedly cancelled: {error}"),
            }
        },
        move |result| {
            let nullable = match result {
                Ok(_) => Purs_Data_Nullable::Data_Nullable_null(),
                Err(error) => Purs_Data_Nullable::Data_Nullable_notNull(error),
            };
            callback.unwrap_func1()(crate::Value::Class(Rc::new(nullable)));
        },
    );
}

fn purust_fs_stats(path: &str) -> std::io::Result<crate::UnknownType> {
    let metadata = std::fs::metadata(path)?;
    Ok(Purs_Node_FS_Stats::purust_stats_box(
        Purs_Node_FS_Stats::purust_stats_from_metadata(&metadata),
    ))
}

fn purust_fs_lstats(path: &str) -> std::io::Result<crate::UnknownType> {
    let metadata = std::fs::symlink_metadata(path)?;
    Ok(Purs_Node_FS_Stats::purust_stats_box(
        Purs_Node_FS_Stats::purust_stats_from_metadata(&metadata),
    ))
}

pub fn Node_FS_Async_accessImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(|path, _mode, callback| {
        let path = purust_fs_path(path);
        purust_fs_dispatch_error_only("access", path.clone(), callback, move || {
            std::fs::metadata(&path)?;
            Ok(())
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_appendFileImpl() -> crate::UnknownType {
    crate::Value::Func4(purust_core::Func4::Shared(Rc::new(
        |path, content, _options, callback| {
            let path = purust_fs_path(path);
            let bytes = match content.resolve() {
                crate::Value::String(text) => {
                    purust_core::purust_string_to_utf8_lossy(text).into_bytes()
                }
                _ => content
                    .unwrap_class::<Rc<Purs_Node_Buffer_Immutable::ImmutableBuffer>>()
                    .bytes(),
            };
            purust_fs_dispatch("open", path.clone(), callback, move || {
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)?;
                file.write_all(&bytes)?;
                Ok(crate::Value::Unit)
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_chmodImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(|path, mode, callback| {
        let path = purust_fs_path(path);
        let mode = mode.unwrap_string();
        purust_fs_dispatch("chmod", path.clone(), callback, move || {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = u32::from_str_radix(&mode, 8).unwrap_or(0o644);
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))?;
            }
            let _ = mode;
            Ok(crate::Value::Unit)
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_chownImpl() -> crate::UnknownType {
    crate::Value::Func4(purust_core::Func4::Shared(Rc::new(
        |path, uid, gid, callback| {
            let path = purust_fs_path(path);
            let uid = uid.unwrap_int() as u32;
            let gid = gid.unwrap_int() as u32;
            purust_fs_dispatch("chown", path.clone(), callback, move || {
                #[cfg(unix)]
                {
                    std::os::unix::fs::chown(&path, Some(uid), Some(gid))?;
                }
                let _ = (uid, gid);
                Ok(crate::Value::Unit)
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_closeImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|fd, callback| {
        let fd = Purs_Node_FS::purust_fd_unbox(&fd);
        purust_fs_dispatch("close", "fd".to_owned(), callback, move || {
            Purs_Node_FS::purust_fd_close(&fd)?;
            Ok(crate::Value::Unit)
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_copyFileImpl() -> crate::UnknownType {
    crate::Value::Func4(purust_core::Func4::Shared(Rc::new(
        |source, destination, mode, callback| {
            let source = purust_fs_path(source);
            let destination = purust_fs_path(destination);
            let exclusive = mode.unwrap_class::<Rc<Purs_Node_FS_Constants::CopyMode>>().0 & 1 != 0;
            let context = format!("{source}' -> '{destination}");
            purust_fs_dispatch("copyfile", context, callback, move || {
                if exclusive && std::path::Path::new(&destination).exists() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "EEXIST",
                    ));
                }
                std::fs::copy(&source, &destination)?;
                Ok(crate::Value::Unit)
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_linkImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(
        |existing, path, callback| {
            let existing = purust_fs_path(existing);
            let path = purust_fs_path(path);
            purust_fs_dispatch("link", path.clone(), callback, move || {
                std::fs::hard_link(&existing, &path)?;
                Ok(crate::Value::Unit)
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_lstatImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, callback| {
        let path = purust_fs_path(path);
        purust_fs_dispatch("lstat", path.clone(), callback, move || purust_fs_lstats(&path));
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_mkdtempImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(
        |prefix, encoding, callback| {
            let prefix = purust_core::purust_string_to_utf8_lossy(&prefix.unwrap_string());
            let encoding = purust_core::purust_string_to_utf8_lossy(&encoding.unwrap_string());
            purust_fs_dispatch("mkdtemp", prefix.clone(), callback, move || {
                let candidate = Purs_Node_FS::purust_mkdtemp(&prefix, &encoding)?;
                Ok(crate::Value::String(Purs_Node_Encoding::purust_encoding_decode(
                    Purs_Node_Encoding::purust_encoding_from_name(&encoding),
                    candidate.as_bytes(),
                )))
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_openImpl() -> crate::UnknownType {
    crate::Value::Func4(purust_core::Func4::Shared(Rc::new(
        |path, flags, mode, callback| {
            let path = purust_fs_path(path);
            let flags = flags.unwrap_string();
            let mode = match mode.unwrap_class::<Rc<Purs_Data_Nullable::Nullable>>().value() {
                Some(value) => value.unwrap_int() as u32,
                None => 0o666,
            };
            purust_fs_dispatch("open", path.clone(), callback, move || {
                let file = Purs_Node_FS::purust_open_options(&flags, mode).open(&path)?;
                Ok(Purs_Node_FS::purust_fd_box(Purs_Node_FS::purust_fd_new(
                    file,
                    flags.starts_with('a'),
                )))
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_readImpl() -> crate::UnknownType {
    crate::Value::Func6(purust_core::Func6::Shared(Rc::new(
        |fd, buffer, offset, length, position, callback| {
            let fd = Purs_Node_FS::purust_fd_unbox(&fd);
            let buffer = buffer
                .unwrap_class::<Rc<Purs_Node_Buffer_Immutable::ImmutableBuffer>>()
                .clone();
            let offset = offset.unwrap_int().max(0) as usize;
            let length = length.unwrap_int().max(0) as usize;
            let position = Purs_Node_FS::purust_nullable_position(&position);
            purust_fs_dispatch("read", "fd".to_owned(), callback, move || {
                let read = Purs_Node_FS::purust_fd_read(&fd, &buffer, offset, length, position)?;
                Ok(crate::mk_int(read))
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_writeImpl() -> crate::UnknownType {
    crate::Value::Func6(purust_core::Func6::Shared(Rc::new(
        |fd, buffer, offset, length, position, callback| {
            let fd = Purs_Node_FS::purust_fd_unbox(&fd);
            let buffer = buffer
                .unwrap_class::<Rc<Purs_Node_Buffer_Immutable::ImmutableBuffer>>()
                .clone();
            let offset = offset.unwrap_int().max(0) as usize;
            let length = length.unwrap_int().max(0) as usize;
            let position = Purs_Node_FS::purust_nullable_position(&position);
            purust_fs_dispatch("write", "fd".to_owned(), callback, move || {
                let written =
                    Purs_Node_FS::purust_fd_write(&fd, &buffer, offset, length, position)?;
                Ok(crate::mk_int(written))
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_readdirImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, callback| {
        let path = purust_fs_path(path);
        purust_fs_dispatch("scandir", path.clone(), callback, move || {
            let names = std::fs::read_dir(&path)?
                .filter_map(|entry| entry.ok())
                .map(|entry| {
                    crate::Value::String(purust_core::purust_string_from_utf8(
                        &entry.file_name().to_string_lossy(),
                    ))
                })
                .collect();
            Ok(purust_core::mk_array(names))
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_readlinkImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, callback| {
        let path = purust_fs_path(path);
        purust_fs_dispatch("readlink", path.clone(), callback, move || {
            let target = std::fs::read_link(&path)?;
            Ok(crate::Value::String(purust_core::purust_string_from_utf8(
                &target.to_string_lossy(),
            )))
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_realpathImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(
        |path, _cache, callback| {
            let path = purust_fs_path(path);
            purust_fs_dispatch("realpath", path.clone(), callback, move || {
                let resolved = std::fs::canonicalize(&path)?;
                Ok(crate::Value::String(purust_core::purust_string_from_utf8(
                    &resolved.to_string_lossy(),
                )))
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_rmImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(
        |path, options, callback| {
            let path = purust_fs_path(path);
            let fields = options.__purust_foreign_object();
            let recursive = fields
                .get("recursive")
                .map(|value| value.unwrap_bool())
                .unwrap_or(false);
            let force = fields
                .get("force")
                .map(|value| value.unwrap_bool())
                .unwrap_or(false);
            purust_fs_dispatch("rm", path.clone(), callback, move || {
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
                    Ok(_) => Ok(crate::Value::Unit),
                    Err(error) if force && error.kind() == std::io::ErrorKind::NotFound => {
                        Ok(crate::Value::Unit)
                    }
                    Err(error) => Err(error),
                }
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_rmdirImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(
        |path, _options, callback| {
            let path = purust_fs_path(path);
            purust_fs_dispatch("rmdir", path.clone(), callback, move || {
                std::fs::remove_dir(&path)?;
                Ok(crate::Value::Unit)
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_statImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, callback| {
        let path = purust_fs_path(path);
        purust_fs_dispatch("stat", path.clone(), callback, move || purust_fs_stats(&path));
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_symlinkImpl() -> crate::UnknownType {
    crate::Value::Func4(purust_core::Func4::Shared(Rc::new(
        |target, path, _kind, callback| {
            let target = purust_fs_path(target);
            let path = purust_fs_path(path);
            purust_fs_dispatch("symlink", path.clone(), callback, move || {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(&target, &path)?;
                }
                Ok(crate::Value::Unit)
            });
            crate::Value::Unit
        },
    )))
}

pub fn Node_FS_Async_truncateImpl() -> crate::UnknownType {
    crate::Value::Func3(purust_core::Func3::Shared(Rc::new(|path, length, callback| {
        let path = purust_fs_path(path);
        let length = length.unwrap_int().max(0) as u64;
        purust_fs_dispatch("ftruncate", path.clone(), callback, move || {
            let file = std::fs::OpenOptions::new().write(true).open(&path)?;
            file.set_len(length)?;
            Ok(crate::Value::Unit)
        });
        crate::Value::Unit
    })))
}

pub fn Node_FS_Async_utimesImpl() -> crate::UnknownType {
    crate::Value::Func4(purust_core::Func4::Shared(Rc::new(
        |path, _access_time, _modified_time, callback| {
            let path = purust_fs_path(path);
            purust_fs_dispatch("utime", path.clone(), callback, move || Ok(crate::Value::Unit));
            crate::Value::Unit
        },
    )))
}
