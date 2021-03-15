use crate::model::user::UserFileUsage;

enum UserFileUsageQuotaKind {
    Unlimited,
    LimitedNumberOfBytes(u64),
}

pub struct UserFileUsageQuota {
    kind: UserFileUsageQuotaKind,
}

impl UserFileUsageQuota {
    pub fn unlimited() -> Self {
        UserFileUsageQuota {
            kind: UserFileUsageQuotaKind::Unlimited,
        }
    }

    pub fn limited_number_of_bytes(max_number_of_bytes: u64) -> Self {
        UserFileUsageQuota {
            kind: UserFileUsageQuotaKind::LimitedNumberOfBytes(max_number_of_bytes),
        }
    }

    pub fn max_number_of_bytes(&self) -> Option<u64> {
        match self.kind {
            UserFileUsageQuotaKind::Unlimited => None,
            UserFileUsageQuotaKind::LimitedNumberOfBytes(max_usage) => Some(max_usage),
        }
    }

    pub fn is_ok(&self, usage: UserFileUsage) -> bool {
        match self.kind {
            UserFileUsageQuotaKind::Unlimited => true,
            UserFileUsageQuotaKind::LimitedNumberOfBytes(max_usage) => {
                max_usage >= usage.to_number_of_bytes()
            }
        }
    }

    /// Returns a number of remaining bytes in the quota.
    ///
    /// If the file usage is not limited in the quota, `None` is returned.
    pub fn remaining_number_of_bytes(&self, current_usage: UserFileUsage) -> Option<u64> {
        if !self.is_ok(current_usage) {
            return Some(0);
        }

        match self.kind {
            UserFileUsageQuotaKind::Unlimited => None,
            UserFileUsageQuotaKind::LimitedNumberOfBytes(max_usage) => {
                Some(max_usage - current_usage.to_number_of_bytes())
            }
        }
    }
}
