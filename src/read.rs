use core::fmt::{self, Debug, Formatter};
#[cfg(feature = "mock")]
use core::marker::PhantomData;
//use either::Either::{self, Left as Own, Right as Ref};
use std::io;

#[cfg(not(feature = "mock"))]
fn unsupported_when_passing_through() -> ! {
    unimplemented!("Unsupported when passing through.");
}
#[cfg(feature = "mock")]
fn unsupported_when_mocking() -> ! {
    unimplemented!("Unsupported when mocking.");
}
//-----
//
// Differentiate "same" type variables/arguments in compile time
const _WITH_CLOSURES: () = {
    let f = || 1usize;
};
// Differentiate with const generic, e.g. string literal?

mod const_gen_from_sister_type {
    struct Stru<const s: usize> {}

    struct Intermediary {}
    impl<const f: usize> From<Stru<f>> for Intermediary {
        fn from(value: Stru<f>) -> Self {
            Self {}
        }
    }

    impl<const t: usize> From<Intermediary> for Stru<t> {
        fn from(value: Intermediary) -> Self {
            Self {}
        }
    }

    fn _test() {
        let s1: Stru<1> = Stru {};
        // Fails:
        //
        //let s2: Stru<2> = s1.into().into();
        let inter: Intermediary = s1.into();
        let s2: Stru<2> = inter.into();
    }

    /*impl <const f: usize, const t: usize> From<Stru<f>> for Stru<t> {
        fn from(value: Stru<f>) -> Self {
            Self {}
        }
    }*/
    /*
    impl <const f: usize, const t: usize> Into<Stru<t>> for Stru<f> {
        fn into(self) -> Stru<t> {
            Stru {}
        }
    }*/
}
// ----
mod cannot_imply_into_type {
    struct Stru {}
    impl From<Stru> for i32 {
        fn from(value: Stru) -> Self {
            1
        }
    }
    fn _test() {
        // let j = 0 + (Stru {}).into();
        //
        //           |
        //
        //           \-> "type must be known at this point"
    }
}

// ----
#[cfg(not(feature = "mock"))]
pub type ReadOwn<R: io::Read> = R;
// - Deref does help, but
// - "consuming" functions that take `self` need to receive an object, not a reference. But, then
//   we'd need to implement ALL methods (and any associated types/constants)!
//   - -> like ReadMut +
//   - -> r.mock_move(XXX) -> a (potentially new) instance of Mockall-based; from an ASSOCIATED type
//     on ReadOwn trait.

#[cfg(not(feature = "mock"))]
pub type ReadRef<'r, R: io::Read> = &'r R;
#[cfg(not(feature = "mock"))]
pub type ReadMut<'r, R: io::Read> = &'r mut R;
//--

#[cfg(feature = "mock")]
pub struct ReadOwn<_R: io::Read> {
    _r: PhantomData<_R>,
}
// Deref, DerefMut
//
// From

#[cfg(feature = "mock")]
#[derive(Copy, Clone)] // because it must be like a shared reference
pub struct ReadRef<'r, R: io::Read> {
    _r: PhantomData<&'r R>,
}

// Incorrect - generic R must be used:
#[cfg(feature="does_not_compile")]
pub type ReadRefDyn<'r, R: io::Read> = &'r dyn io::Read;

#[cfg(feature = "mock")]
pub struct ReadMut<'r, R: io::Read> {
    _r: PhantomData<&'r R>,
}

// We do NOT need type switching to mock `&'a dyn TargetTrait` or ``&'a mut dyn TargetTrait` -
// because those can be replaced with any object-safe implementation.
//
// BUT we can have this, to make use cases similar.
pub type ReadDyn<'a> = &'a dyn io::Read;
pub type ReadMutDyn<'a> = &'a mut dyn io::Read;
//-----

/* */
// Thanks to [`impl<R: Read + ?Sized> Read for &mut R
// `](https://doc.rust-lang.org/nightly/std/io/trait.Read.html#impl-Read-for-%26mut+R) we do NOT
// need to specialize on/handle (mutable) references to [io::Read], neither [std::boxed::Box] of
// [io::Read].
#[cfg(not(feature = "mock"))]
type PassedReadBranch<R: io::Read> = R;
#[cfg(feature = "mock")]
type PassedReadBranch<R: io::Read> = (!, PhantomData<R>);

#[cfg(not(feature = "mock"))]
type MockedReadBranch = !;
#[cfg(feature = "mock")]
type MockedReadBranch = MoReadBox;

pub enum MoRead<R: io::Read> {
    Pass(PassedReadBranch<R>),
    Mock(MockedReadBranch),
}

impl<R: io::Read> MoRead<R> {
    pub fn new_pass(r: R) -> Self {
        #[cfg(not(feature = "mock"))]
        return Self::Pass(r);
        #[cfg(feature = "mock")]
        unsupported_when_mocking();
    }

    pub fn new_mock() -> Self {
        #[cfg(feature = "mock")]
        return Self::Mock(MoReadBox::default());
        #[cfg(not(feature = "mock"))]
        unsupported_when_passing_through();
    }

    pub fn new_mock_from(_mocked_read_box: MoReadBox) -> Self {
        #[cfg(feature = "mock")]
        return Self::Mock(_mocked_read_box);
        #[cfg(not(feature = "mock"))]
        unsupported_when_passing_through();
    }

    /// Use in either Pass or Mock mode. However, do NOT depend on the type of this function's
    /// result. So, for example, do NOT use the result:
    /// - storing it in variables (other than with inferred type), or
    /// - passing it to functions (other than generic), or returning from functions, or
    /// - depend on it being [Debug] or any trait other than the given trait (Read).
    #[cfg(not(feature = "mock"))]
    pub fn get(&mut self) -> &mut R {
        match *self {
            Self::Pass(ref mut passed) => {
                return passed;
            }
            Self::Mock(_) => {
                unreachable!();
            }
        }
    }
    #[cfg(feature = "mock")]
    // CanNOT return &mut dyn ReadDebuggable - because ReadDebuggable is NOT (auto)implemented for
    // Box<dyn ReadDebuggable>.
    pub fn get(&mut self) -> &mut Box<dyn ReadDebuggable> {
        match *self {
            Self::Pass(_) => {
                unreachable!();
            }
            Self::Mock(ref mut mocked) => {
                return &mut *mocked.r.as_mut().unwrap();
            }
        }
    }

    /// Use only for pass-through.
    pub fn pass(&mut self) -> &mut R {
        match *self {
            Self::Pass(ref mut _passed) => {
                #[cfg(not(feature = "mock"))]
                return _passed;
                #[cfg(feature = "mock")]
                unreachable!();
            }
            Self::Mock(_) => {
                #[cfg(feature = "mock")]
                unsupported_when_mocking();
                #[cfg(not(feature = "mock"))]
                unreachable!();
            }
        }
    }

    /// Use only to move out, and only for pass-through.
    pub fn pass_out(self) -> R {
        match self {
            Self::Pass(_passed) => {
                #[cfg(not(feature = "mock"))]
                return _passed;
                #[cfg(feature = "mock")]
                unreachable!();
            }
            Self::Mock(_) => {
                #[cfg(feature = "mock")]
                unsupported_when_mocking();
                #[cfg(not(feature = "mock"))]
                unreachable!();
            }
        }
    }
}

// @TODO impl Deref

// NOT including Clone, because reading mutates the reader (like a &mut[u8]), so (like a mutable
// reference or slice) they can't be cloned (in general).
pub trait ReadDebuggable: std::io::Read + core::fmt::Debug {}
impl<T: std::io::Read + core::fmt::Debug> ReadDebuggable for T {}

impl<R: ReadDebuggable> Debug for MoRead<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Pass(ref passed) => f.write_fmt(format_args!("Mread::Pass({passed:?})")),
            Self::Mock(ref mocked) => f.write_fmt(format_args!("Mread::Mocked({mocked:?})")),
        }
    }
}

#[cfg(not(feature = "mock"))]
const _: () = {
    assert!(
        core::mem::size_of::<MoRead<io::Stdin>>() == core::mem::size_of::<io::Stdin>(),
        "The pass-through mode must be zero cost."
    );
};

#[derive(Debug, Default)]
pub struct MoReadBox {
    // An Option, so that we can flexibly move it in & out, or initiate it later/lazily.
    r: Option<Box<dyn ReadDebuggable>>,
    // @TODO more
}
impl MoReadBox {
    pub fn new_from(box_read: Box<dyn ReadDebuggable>) -> Self {
        Self { r: Some(box_read) }
    }
}
// -------
// ------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let a = [0, 1, 2u8];
        let mut s = &a[..];
        if true {
            let mut _dyn_s: &dyn std::io::Read = &mut s; // ok
        } else {
            //let mut dyn_s: &dyn std::io::Read = s; // NOT ok!
        }
    }
}
/* */
