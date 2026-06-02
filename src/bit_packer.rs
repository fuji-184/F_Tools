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

ftest::test!(bit_packer_tests, {
    test_u16_packing {
        let high: u8 = 171;
        let low: u8 = 205;
        let packed: u16 = BitPacker::pack(high, low);
        assert_eq!(packed, 43981);

        let (unpacked_high, unpacked_low) = BitPacker::unpack(packed);
        assert_eq!(unpacked_high, high);
        assert_eq!(unpacked_low, low);
    }

    test_u32_packing {
        let high: u16 = 4660;
        let low: u16 = 22136;
        let packed: u32 = BitPacker::pack(high, low);
        assert_eq!(packed, 305419896);

        let (unpacked_high, unpacked_low) = BitPacker::unpack(packed);
        assert_eq!(unpacked_high, high);
        assert_eq!(unpacked_low, low);
    }

    test_u64_packing {
        let high: u32 = 287454020;
        let low: u32 = 1432778632;
        let packed: u64 = BitPacker::pack(high, low);
        assert_eq!(packed, 1234605616436508552);

        let (unpacked_high, unpacked_low) = BitPacker::unpack(packed);
        assert_eq!(unpacked_high, high);
        assert_eq!(unpacked_low, low);
    }

    test_u128_packing {
        let high: u64 = 123456789012345678;
        let low: u64 = 987654321098765432;
        let packed: u128 = BitPacker::pack(high, low);
        assert_eq!(packed, 2277375791072698124611201734874281080);

        let (unpacked_high, unpacked_low) = BitPacker::unpack(packed);
        assert_eq!(unpacked_high, high);
        assert_eq!(unpacked_low, low);
    }
});