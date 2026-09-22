use tactica_uuid_kinds::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Principal {
    User(UserId),
}

impl Principal {
    pub fn from_jwt_claims(claims: &crate::jwt::Claims) -> Result<Self, String> {
        match claims.tty.as_str() {
            "user" => {
                let user_id = UserId::try_from(claims.sub.clone()).map_err(|e| e.to_string())?;

                Ok(Self::User(user_id))
            }

            _ => Err(format!("Unknown principal type: {}", claims.tty)),
        }
    }
}
