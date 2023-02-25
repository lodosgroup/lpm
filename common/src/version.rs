use crate::de_required_field;

use json::JsonValue;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct VersionStruct {
    pub readable_format: String,
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub tag: Option<String>,
}

impl VersionStruct {
    pub fn compare(&self, to: &VersionStruct) -> Ordering {
        match self.major.cmp(&to.major) {
            std::cmp::Ordering::Less => Ordering::Less,
            std::cmp::Ordering::Greater => Ordering::Greater,
            std::cmp::Ordering::Equal => match self.minor.cmp(&to.minor) {
                std::cmp::Ordering::Less => Ordering::Less,
                std::cmp::Ordering::Greater => Ordering::Greater,
                std::cmp::Ordering::Equal => match self.patch.cmp(&to.patch) {
                    std::cmp::Ordering::Less => Ordering::Less,
                    std::cmp::Ordering::Greater => Ordering::Greater,
                    std::cmp::Ordering::Equal => {
                        if to.tag == self.tag {
                            Ordering::Equal
                        } else {
                            // If major.minor.patch version is same but
                            // tag is different, then we will consider it as
                            // higher version since tags are not standardized.
                            Ordering::Greater
                        }
                    }
                },
            },
        }
    }
}

impl json::Deserialize for VersionStruct {
    type Error = String;

    fn from_json_object(json: &json::JsonValue) -> Result<Self, Self::Error> {
        let object = Self {
            readable_format: de_required_field!(
                json["readable_format"].to_string(),
                "readable_format"
            ),
            major: de_required_field!(json["major"].as_u8(), "major"),
            minor: de_required_field!(json["minor"].as_u8(), "minor"),
            patch: de_required_field!(json["patch"].as_u8(), "patch"),
            tag: json["tag"].to_string(),
        };

        Ok(object)
    }

    fn from_json_array(json: &json::JsonValue) -> Result<Vec<Self>, Self::Error> {
        let mut object_array = vec![];
        match json {
            JsonValue::Array(array) => {
                for item in array {
                    let object = Self::from_json_object(item)?;
                    object_array.push(object);
                }
            }
            _ => return Err("Wrong input, expected an array".to_string()),
        };

        Ok(object_array)
    }
}

#[cfg(test)]
mod tests {
    use super::VersionStruct;

    use std::cmp::Ordering;

    #[test]
    fn test_version_comparison() {
        let mut x = VersionStruct {
            readable_format: "1.0.0".to_string(),
            major: 1,
            minor: 0,
            patch: 0,
            tag: None,
        };

        let mut y = VersionStruct {
            readable_format: "1.0.1".to_string(),
            major: 1,
            minor: 0,
            patch: 1,
            tag: None,
        };

        assert_eq!(x.compare(&y), Ordering::Less);

        x.minor = 2;
        x.readable_format = "1.2.0".to_string();
        y.minor = 1;
        y.readable_format = "1.1.1".to_string();
        assert_eq!(x.compare(&y), Ordering::Greater);

        x.patch = 1;
        x.readable_format = "1.2.1".to_string();
        y.minor = 2;
        y.readable_format = "1.2.1".to_string();
        assert_eq!(x.compare(&y), Ordering::Equal);

        x.tag = Some("-beta".to_string());
        assert_eq!(x.compare(&y), Ordering::Greater);

        x.tag = None;
        y.tag = Some("-beta".to_string());
        assert_eq!(x.compare(&y), Ordering::Greater);

        x.tag = Some("-alpha1".to_string());
        y.tag = Some("-alpha1".to_string());
        assert_eq!(x.compare(&y), Ordering::Equal);
    }
}
