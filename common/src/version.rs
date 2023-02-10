use crate::de_required_field;

use json::JsonValue;

#[derive(Debug, Clone)]
pub struct VersionStruct {
    pub readable_format: String,
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub tag: Option<String>,
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
