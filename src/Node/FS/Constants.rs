// Access and copy modes from Node's `fs.constants`.
use std::rc::Rc;

pub struct AccessMode(pub i32);
pub struct CopyMode(pub i32);

pub fn Node_FS_Constants_f_OK() -> Rc<AccessMode> {
    Rc::new(AccessMode(0))
}

pub fn Node_FS_Constants_r_OK() -> Rc<AccessMode> {
    Rc::new(AccessMode(4))
}

pub fn Node_FS_Constants_w_OK() -> Rc<AccessMode> {
    Rc::new(AccessMode(2))
}

pub fn Node_FS_Constants_x_OK() -> Rc<AccessMode> {
    Rc::new(AccessMode(1))
}

pub fn Node_FS_Constants_copyFile_EXCL() -> Rc<CopyMode> {
    Rc::new(CopyMode(1))
}

pub fn Node_FS_Constants_copyFile_FICLONE() -> Rc<CopyMode> {
    Rc::new(CopyMode(2))
}

pub fn Node_FS_Constants_copyFile_FICLONE_FORCE() -> Rc<CopyMode> {
    Rc::new(CopyMode(4))
}

pub fn Node_FS_Constants_appendCopyMode() -> crate::UnknownType {
    crate::Value::Func2(purust_core::Func2::Shared(Rc::new(|left, right| {
        let left = left.unwrap_class::<Rc<CopyMode>>().0;
        let right = right.unwrap_class::<Rc<CopyMode>>().0;
        crate::Value::Class(Rc::new(CopyMode(left | right)))
    })))
}
