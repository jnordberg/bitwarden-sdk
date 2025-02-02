use std::str::FromStr;

use base64::Engine;
use uuid::Uuid;

use crate::{
    client::encryption_settings::SymmetricCryptoKey, error::AccessTokenInvalidError,
    util::BASE64_ENGINE,
};

pub struct AccessToken {
    pub service_account_id: Uuid,
    pub client_secret: String,
    pub encryption_key: SymmetricCryptoKey,
}

impl FromStr for AccessToken {
    type Err = crate::error::Error;

    fn from_str(key: &str) -> std::result::Result<Self, Self::Err> {
        let (first_part, encryption_key) =
            key.split_once(':').ok_or(AccessTokenInvalidError::NoKey)?;

        let [version, service_account_id, client_secret]: [&str; 3] = first_part
            .split('.')
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| AccessTokenInvalidError::WrongParts)?;

        if version != "0" {
            return Err(AccessTokenInvalidError::WrongVersion.into());
        }

        let Ok(service_account_id) = service_account_id.parse() else { return Err(AccessTokenInvalidError::InvalidUuid.into()) };

        let encryption_key = BASE64_ENGINE
            .decode(encryption_key)
            .map_err(AccessTokenInvalidError::InvalidBase64)?;
        let encryption_key: [u8; 16] = encryption_key.try_into().map_err(|e: Vec<_>| {
            AccessTokenInvalidError::InvalidBase64Length {
                expected: 16,
                got: e.len(),
            }
        })?;
        let encryption_key =
            crate::crypto::stretch_key(encryption_key, "accesstoken", Some("sm-access-token"));

        Ok(AccessToken {
            service_account_id: service_account_id,
            client_secret: client_secret.to_owned(),
            encryption_key,
        })
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn can_decode_access_token() {
        use crate::client::AccessToken;
        use std::str::FromStr;

        let access_token = "0.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ==";
        let token = AccessToken::from_str(access_token).unwrap();

        assert_eq!(
            &token.service_account_id.to_string(),
            "ec2c1d46-6a4b-4751-a310-af9601317f2d"
        );
        assert_eq!(token.client_secret, "C2IgxjjLF7qSshsbwe8JGcbM075YXw");
        assert_eq!(token.encryption_key.to_base64(), "H9/oIRLtL9nGCQOVDjSMoEbJsjWXSOCb3qeyDt6ckzS3FhyboEDWyTP/CQfbIszNmAVg2ExFganG1FVFGXO/Jg==");
    }

    #[test]
    fn malformed_tokens() {
        use crate::client::AccessToken;
        use std::str::FromStr;

        // Encryption key without base64 padding, we generate it with padding but ignore it when decoding
        let t = "0.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ";
        assert!(AccessToken::from_str(t).is_ok());

        // Invalid version
        let t = "1.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ==";
        assert!(AccessToken::from_str(t).is_err());

        // Invalid splits
        let t = "0.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw.X8vbvA0bduihIDe/qrzIQQ==";
        assert!(AccessToken::from_str(t).is_err());

        let t = "ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ==";
        assert!(AccessToken::from_str(t).is_err());

        // Invalid base64
        let t = "1.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe9qrzIQQ==";
        assert!(AccessToken::from_str(t).is_err());
    }
}
