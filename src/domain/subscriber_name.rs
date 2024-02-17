use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        println!("String: {}", s);
        if s.trim().is_empty() {
            return Err(String::from(
                "Subscriber name has to have at least one non-empty character",
            ));
        }

        if s.graphemes(true).count() > 256 {
            return Err(String::from("Subscriber name max length is 255 characters"));
        }

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

        // false if contains a forbidden character, true otherwise
        if s.chars().any(|c| forbidden_characters.contains(&c)) {
            return Err(format!(
                "Subscriber name can't contain a forbidden character: {:?}",
                forbidden_characters
            ));
        }

        Ok(Self(s))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_256_grapheme_name_valid() {
        let name = "a̐".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn test_longer_than_256_grapheme_name_invalid() {
        let name = "a̐".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn test_whitespace_only_invalid() {
        let name = String::from(" ");
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn test_empty_string_invalid() {
        let name = String::from("");
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn test_name_with_invalid_characters_are_invalid() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn test_correct_name_valid() {
        let name = String::from("Ferran Palma");
        assert_ok!(SubscriberName::parse(name));
    }
}
