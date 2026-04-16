use borsh::BorshSerialize;

pub(crate) trait BorshSerializeExt: BorshSerialize {
    fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        borsh::to_vec(self)
    }
}

impl<T: BorshSerialize + ?Sized> BorshSerializeExt for T {}
