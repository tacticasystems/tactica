//! Unit permissions granted by roles. Ranks never grant permissions.

/// Stable bit positions stored in role permission masks and sent over the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum Permission {
    /// Implicitly grants every permission, including permissions added later.
    Administrator = 1 << 0,
    /// Manage the unit's profile and settings.
    ManageUnit = 1 << 1,
    ManageRoles = 1 << 2,
    AssignRoles = 1 << 3,
    ManageRanks = 1 << 4,
    AssignRanks = 1 << 5,
    /// Edit display fields on member profiles; does not grant membership removal.
    ManageMembers = 1 << 6,
}

impl Permission {
    #[must_use]
    pub const fn bits(self) -> i64 {
        self as i64
    }
}

/// A validated set of explicit role permissions. Administrator is expanded at
/// evaluation time, so stored masks do not need rewriting when a bit is added.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Permissions(i64);

impl Permissions {
    pub const KNOWN_BITS: i64 = Permission::Administrator.bits()
        | Permission::ManageUnit.bits()
        | Permission::ManageRoles.bits()
        | Permission::AssignRoles.bits()
        | Permission::ManageRanks.bits()
        | Permission::AssignRanks.bits()
        | Permission::ManageMembers.bits();

    pub const ADMINISTRATOR: Self = Self(Permission::Administrator.bits());

    /// Reject negative masks and undefined bits, including for administrators.
    #[must_use]
    pub const fn from_bits(bits: i64) -> Option<Self> {
        if bits & !Self::KNOWN_BITS == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn bits(self) -> i64 {
        self.0
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn allows(self, permission: Permission) -> bool {
        self.0 & Permission::Administrator.bits() != 0 || self.0 & permission.bits() != 0
    }

    /// Whether the caller holds a set of permission bits. Holding all
    /// ordinary permissions does not itself grant the Administrator bit.
    #[must_use]
    pub const fn covers(self, role: Self) -> bool {
        self.allows(Permission::Administrator) || role.0 & !self.0 == 0
    }
}

impl From<Permission> for Permissions {
    fn from(permission: Permission) -> Self {
        Self(permission.bits())
    }
}

#[cfg(test)]
mod tests {
    use super::{Permission, Permissions};

    #[test]
    fn masks_validate_known_bits_and_administrator_implies_all_permissions() {
        assert_eq!(Permissions::KNOWN_BITS, 127);
        for invalid in [-1, i64::MIN, 128, 129, i64::MAX] {
            assert!(Permissions::from_bits(invalid).is_none());
        }
        let all = [
            Permission::Administrator,
            Permission::ManageUnit,
            Permission::ManageRoles,
            Permission::AssignRoles,
            Permission::ManageRanks,
            Permission::AssignRanks,
            Permission::ManageMembers,
        ];
        for permission in all {
            assert!(Permissions::ADMINISTRATOR.allows(permission));
            assert!(!Permissions::default().allows(permission));
        }
    }

    #[test]
    fn role_union_does_not_turn_all_ordinary_permissions_into_administrator() {
        let management =
            Permissions::from(Permission::ManageRoles).union(Permission::AssignRoles.into());
        assert!(management.allows(Permission::ManageRoles));
        assert!(management.allows(Permission::AssignRoles));
        assert!(!management.allows(Permission::ManageMembers));
        assert!(management.covers(Permission::ManageRoles.into()));
        assert!(!management.covers(Permission::ManageUnit.into()));
        let ordinary = Permissions::from_bits(126).expect("all ordinary permissions");
        assert!(!ordinary.allows(Permission::Administrator));
        assert!(!ordinary.covers(Permissions::ADMINISTRATOR));
        assert!(Permissions::ADMINISTRATOR.covers(ordinary));
    }
}
