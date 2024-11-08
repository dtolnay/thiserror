use core::fmt::{
    self, Binary, Debug, Display, LowerExp, LowerHex, Octal, Pointer, UpperExp, UpperHex,
};

pub struct Var<'a, T: ?Sized>(pub &'a T);

/// Pointer is the only one for which there is a difference in behavior between
/// `Var<'a, T>` vs `&'a T`.
impl<'a, T: Pointer + ?Sized> Pointer for Var<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Pointer::fmt(self.0, formatter)
    }
}

impl<'a, T: Binary + ?Sized> Binary for Var<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Binary::fmt(self.0, formatter)
    }
}

impl<'a, T: Debug + ?Sized> Debug for Var<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.0, formatter)
    }
}

impl<'a, T: Display + ?Sized> Display for Var<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self.0, formatter)
    }
}

impl<'a, T: LowerExp + ?Sized> LowerExp for Var<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        LowerExp::fmt(self.0, formatter)
    }
}

impl<'a, T: LowerHex + ?Sized> LowerHex for Var<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        LowerHex::fmt(self.0, formatter)
    }
}

impl<'a, T: Octal + ?Sized> Octal for Var<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Octal::fmt(self.0, formatter)
    }
}

impl<'a, T: UpperExp + ?Sized> UpperExp for Var<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        UpperExp::fmt(self.0, formatter)
    }
}

impl<'a, T: UpperHex + ?Sized> UpperHex for Var<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        UpperHex::fmt(self.0, formatter)
    }
}
