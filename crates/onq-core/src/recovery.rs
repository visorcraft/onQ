use base64::Engine;
use bip39::{Language, Mnemonic};

use crate::error::{CoreError, CoreResult};

/// Generate a fresh 24-word English BIP39 recovery phrase.
pub fn generate_phrase() -> String {
    Mnemonic::generate_in(Language::English, 24)
        .expect("24 words is a valid BIP39 word count")
        .to_string()
}

/// Derive the stable master passphrase represented by a BIP39 phrase.
pub fn phrase_to_passphrase(phrase: &str) -> CoreResult<String> {
    let mnemonic = parse_phrase(phrase)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(mnemonic.to_seed("")))
}

/// Validate that `phrase` is an English BIP39 mnemonic with a valid checksum.
pub fn validate_phrase(phrase: &str) -> CoreResult<()> {
    parse_phrase(phrase).map(|_| ())
}

fn parse_phrase(phrase: &str) -> CoreResult<Mnemonic> {
    let mnemonic = Mnemonic::parse_in(Language::English, phrase)
        .map_err(|error| CoreError::Encryption(format!("invalid recovery phrase: {error}")))?;
    if mnemonic.word_count() != 24 {
        return Err(CoreError::Encryption(
            "recovery phrase must contain 24 words".into(),
        ));
    }
    Ok(mnemonic)
}

#[cfg(test)]
mod tests {
    use super::{generate_phrase, phrase_to_passphrase, validate_phrase};

    #[test]
    fn generated_phrase_is_valid_and_has_24_words() {
        let phrase = generate_phrase();

        assert_eq!(phrase.split_whitespace().count(), 24);
        assert!(validate_phrase(&phrase).is_ok());
    }

    #[test]
    fn same_phrase_derives_same_passphrase() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

        let first = phrase_to_passphrase(phrase).unwrap();
        let second = phrase_to_passphrase(phrase).unwrap();

        assert_eq!(first, second);
        assert_eq!(
            first,
            "QIsoXBI4NgBPS4hCyJMkwfATgkUMDUOa80W6f8Saz3BUicb8d9vU49wd2MxryfBD24raHiQ8Sg6vspDTmUgIQA=="
        );
    }

    #[test]
    fn shorter_valid_bip39_phrase_is_rejected() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        assert!(validate_phrase(phrase).is_err());
        assert!(phrase_to_passphrase(phrase).is_err());
    }

    #[test]
    fn invalid_phrase_is_rejected() {
        assert!(validate_phrase("not a valid recovery phrase").is_err());
        assert!(phrase_to_passphrase("abandon abandon abandon").is_err());
    }
}
