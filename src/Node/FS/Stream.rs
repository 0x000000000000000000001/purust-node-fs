// File-system streams, built on the node-streams native model.
use std::rc::Rc;

use Purs_Node_Stream::purust_stream_box;

fn path_of(value: &crate::UnknownType) -> String {
    purust_core::purust_string_to_utf8_lossy(&value.unwrap_string())
}

pub fn Node_FS_Stream_createReadStreamImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|path| {
        purust_stream_box(Purs_Node_Stream::purust_readable_from_file(&path_of(&path)))
    })))
}

pub fn Node_FS_Stream_createReadStreamOptsImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, _options| {
        purust_stream_box(Purs_Node_Stream::purust_readable_from_file(&path_of(&path)))
    })))
}

pub fn Node_FS_Stream_fdCreateReadStreamImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|fd| {
        let fd = Purs_Node_FS::purust_fd_unbox(&fd);
        let mut file = fd.file.lock().unwrap();
        use std::io::{Read, Seek, SeekFrom};
        let _ = file.seek(SeekFrom::Start(0));
        let mut bytes = Vec::new();
        let _ = file.read_to_end(&mut bytes);
        purust_stream_box(Purs_Node_Stream::purust_readable_from_bytes(bytes))
    })))
}

pub fn Node_FS_Stream_fdCreateReadStreamOptsImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|fd, _options| {
        let fd = Purs_Node_FS::purust_fd_unbox(&fd);
        let mut file = fd.file.lock().unwrap();
        use std::io::{Read, Seek, SeekFrom};
        let _ = file.seek(SeekFrom::Start(0));
        let mut bytes = Vec::new();
        let _ = file.read_to_end(&mut bytes);
        purust_stream_box(Purs_Node_Stream::purust_readable_from_bytes(bytes))
    })))
}

pub fn Node_FS_Stream_createWriteStreamImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|path| {
        purust_stream_box(Purs_Node_Stream::purust_writable_to_file(&path_of(&path)))
    })))
}

pub fn Node_FS_Stream_createWriteStreamOptsImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|path, _options| {
        purust_stream_box(Purs_Node_Stream::purust_writable_to_file(&path_of(&path)))
    })))
}

pub fn Node_FS_Stream_fdCreateWriteStreamImpl() -> crate::UnknownType {
    crate::Value::Func1(purust_core::Func1::Shared(Rc::new(|_fd| {
        purust_stream_box(Purs_Node_Stream::purust_writable_new())
    })))
}

pub fn Node_FS_Stream_fdCreateWriteStreamOptsImpl() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|_fd, _options| {
        purust_stream_box(Purs_Node_Stream::purust_writable_new())
    })))
}
