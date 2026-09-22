use tactica_uuid_kinds::UserId;

use crate::jwt::TokenType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Principal {
    User(UserId),
    Service(String),
}

impl Principal {
    pub fn from_jwt_claims(claims: &crate::jwt::Claims) -> Result<Self, String> {
        match claims.tty {
            TokenType::User => {
                let user_id = UserId::try_from(claims.sub.clone()).map_err(|e| e.to_string())?;

                Ok(Self::User(user_id))
            }
            TokenType::Service => Ok(Self::Service(claims.sub.clone())),
        }
    }
}
