pub trait Packable {
    type Half;
    fn from_parts(high: Self::Half, low: Self::Half) -> Self;
    fn to_parts(self) -> (Self::Half, Self::Half);
}

macro_rules! impl_bit_packer {
    ($t:ty, $half:ty, $shift:expr) => {
        impl Packable for $t {
            type Half = $half;
            #[inline(always)]
            fn from_parts(high: Self::Half, low: Self::Half) -> Self {
                ((high as $t) << $shift) | (low as $t)
            }
            #[inline(always)]
            fn to_parts(self) -> (Self::Half, Self::Half) {
                ((self >> $shift) as $half, self as $half)
            }
        }
    };
}

impl_bit_packer!(u16, u8, 8);
impl_bit_packer!(u32, u16, 16);
impl_bit_packer!(u64, u32, 32);
impl_bit_packer!(u128, u64, 64);

pub struct BitPacker;

impl BitPacker {
    #[inline]
    pub fn pack<T: Packable>(high: T::Half, low: T::Half) -> T {
        T::from_parts(high, low)
    }

    #[inline]
    pub fn unpack<T: Packable>(packed: T) -> (T::Half, T::Half) {
        T::to_parts(packed)
    }
}