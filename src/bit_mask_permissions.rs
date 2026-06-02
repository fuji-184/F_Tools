
/*
High-performance bitmask utility for zero-cost permission and privilege management.

This module provides an inlineable API to perform granular bitwise mutations and checks 
over unsigned 64-bit flag integers. It supports typical authorization workflows including 
granting, revoking, toggling, and multi-flag intersecting (all-or-any matching) without 
runtime allocations, enforcing compact and cache-friendly operational access control.
*/

pub struct PermissionMask;

impl PermissionMask {
    #[inline]
    pub fn grant(mask: u64, permission: u64) -> u64 {
        mask | permission
    }

    #[inline]
    pub fn grant_many(mut mask: u64, perms: &[u64]) -> u64 {
        for p in perms {
            mask |= p;
        }
        mask
    }
    
    #[inline]
    pub fn toggle(mask: u64, permission: u64) -> u64 {
        mask ^ permission
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

ftest::test!(permission_mask_tests, {
    test_grant_and_revoke {
        let mut mask = permissions::NONE;

        mask = PermissionMask::grant(mask, permissions::READ);
        assert!(PermissionMask::has_all(mask, permissions::READ));

        mask = PermissionMask::grant(mask, permissions::WRITE);
        assert!(PermissionMask::has_all(mask, permissions::EDITOR));

        mask = PermissionMask::revoke(mask, permissions::WRITE);
        assert!(!PermissionMask::has_all(mask, permissions::WRITE));
        assert!(PermissionMask::has_all(mask, permissions::READ));
    }

    test_grant_many {
        let perms = [permissions::READ, permissions::WRITE, permissions::EXECUTE];
        let mask = PermissionMask::grant_many(permissions::NONE, &perms);

        assert!(PermissionMask::has_all(mask, permissions::READ | permissions::WRITE | permissions::EXECUTE));
        assert!(!PermissionMask::has_all(mask, permissions::DELETE));
    }

    test_toggle {
        let mut mask = permissions::NONE;

        mask = PermissionMask::toggle(mask, permissions::READ);
        assert!(PermissionMask::has_all(mask, permissions::READ));

        mask = PermissionMask::toggle(mask, permissions::READ);
        assert!(!PermissionMask::has_all(mask, permissions::READ));
    }

    test_has_any {
        let mask = permissions::EDITOR;

        assert!(PermissionMask::has_any(mask, permissions::READ | permissions::DELETE));
        assert!(!PermissionMask::has_any(mask, permissions::DELETE | permissions::EXECUTE));
    }

    test_admin_permissions {
        let mask = permissions::ADMIN;

        assert!(PermissionMask::has_all(mask, permissions::READ));
        assert!(PermissionMask::has_all(mask, permissions::WRITE));
        assert!(PermissionMask::has_all(mask, permissions::DELETE));
        assert!(PermissionMask::has_all(mask, permissions::EXECUTE));
    }
});