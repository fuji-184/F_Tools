pub struct PermissionMask;

impl PermissionMask {
    #[inline]
    pub fn grant(mask: u64, permission: u64) -> u64 {
        mask | permission
    }

    #[inline]
    pub fn revoke(mask: u64, permission: u64) -> u64 {
        mask & !permission
    }

    #[inline]
    pub fn has_all(mask: u64, required: u64) -> bool {
        (mask & required) == required
    }

    #[inline]
    pub fn has_any(mask: u64, any_of: u64) -> bool {
        (mask & any_of) != 0
    }
}

pub mod permissions {
    pub const NONE: u64   = 0;          // 0000
    pub const READ: u64   = 1 << 0;     // 0001 (1)
    pub const WRITE: u64  = 1 << 1;     // 0010 (2)
    pub const DELETE: u64 = 1 << 2;     // 0100 (4)
    pub const EXECUTE: u64 = 1 << 3;    // 1000 (8)
    
    pub const EDITOR: u64 = READ | WRITE;
    pub const ADMIN: u64  = READ | WRITE | DELETE | EXECUTE;
}