use crate::{IntegerMachineType, RealMachineType};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumericType {
    Integer(IntegerMachineType),
    Real(RealMachineType),
}

impl NumericType {
    pub(super) fn as_real(&self) -> RealMachineType {
        match self {
            NumericType::Integer(i) => *i as RealMachineType,
            NumericType::Real(r) => *r,
        }
    }
    pub(super) fn as_int(&self) -> IntegerMachineType {
        match self {
            NumericType::Integer(i) => *i,
            NumericType::Real(r) => *r as IntegerMachineType,
        }
    }
}

impl Display for NumericType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericType::Integer(i) => Display::fmt(&i, f),
            NumericType::Real(r) => Display::fmt(&r, f),
        }
    }
}

impl Add for NumericType {
    type Output = NumericType;

    fn add(self, rhs: Self) -> Self::Output {
        if let (NumericType::Integer(i1), NumericType::Integer(i2)) = (self, rhs) {
            NumericType::Integer(i1 + i2)
        } else {
            NumericType::Real(self.as_real() + rhs.as_real())
        }
    }
}

impl Sub for NumericType {
    type Output = NumericType;

    fn sub(self, rhs: Self) -> Self::Output {
        if let (NumericType::Integer(i1), NumericType::Integer(i2)) = (self, rhs) {
            NumericType::Integer(i1 - i2)
        } else {
            NumericType::Real(self.as_real() - rhs.as_real())
        }
    }
}

impl Mul for NumericType {
    type Output = NumericType;

    fn mul(self, rhs: Self) -> Self::Output {
        if let (NumericType::Integer(i1), NumericType::Integer(i2)) = (self, rhs) {
            NumericType::Integer(i1 * i2)
        } else {
            NumericType::Real(self.as_real() * rhs.as_real())
        }
    }
}

impl Neg for NumericType {
    type Output = NumericType;

    fn neg(self) -> Self::Output {
        match self {
            NumericType::Integer(i) => NumericType::Integer(-i),
            NumericType::Real(r) => NumericType::Real(-r),
        }
    }
}
