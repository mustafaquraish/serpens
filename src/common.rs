use std::cell::RefCell;
use std::rc::Rc;

pub type Ref<T> = Rc<RefCell<T>>;

macro_rules! get {
    ($val:expr) => {
        &*$val.borrow()
    };
}
pub(crate) use get;

macro_rules! make {
    ($val:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new($val))
    };
}
pub(crate) use make;

