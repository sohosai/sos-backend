use std::fmt::{self, Display};
use std::str::FromStr;

use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    Administrator,
    Committee,
    General,
}

#[derive(Debug, Error, Clone)]
#[error("invalid role")]
pub struct ParseRoleError {
    _priv: (),
}

impl FromStr for Role {
    type Err = ParseRoleError;
    fn from_str(s: &str) -> Result<Role, Self::Err> {
        match s {
            "administrator" => Ok(Role::Administrator),
            "committee" => Ok(Role::Committee),
            "general" => Ok(Role::General),
            _ => Err(ParseRoleError { _priv: () }),
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Administrator => f.write_str("administrator"),
            Role::Committee => f.write_str("committee"),
            Role::General => f.write_str("general"),
        }
    }
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::Role;
    use std::str::FromStr;

    #[test]
    fn test_display_fromstr() {
        fn check(role: Role) {
            assert_eq!(Role::from_str(&role.to_string()).unwrap(), role);
        }
        check(Role::Administrator);
        check(Role::Committee);
        check(Role::General);
    }

    #[test]
    fn test_fromstr_display() {
        fn check(s: &str) {
            assert_eq!(Role::from_str(s).unwrap().to_string(), s);
        }
        check("administrator");
        check("committee");
        check("general");
    }
}
