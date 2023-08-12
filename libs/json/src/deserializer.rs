use crate::json_value::JsonValue;

pub trait Deserialize {
    type Error;

    fn from_json_object(json: &JsonValue) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn from_json_array(json: &JsonValue) -> Result<Vec<Self>, Self::Error>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::Json;

    use std::collections::BTreeMap;

    macro_rules! unwrap_option_or_err {
        ($val: expr, $err: expr) => {
            match $val {
                Some(val) => val,
                None => return Err($err.to_string()),
            }
        };
    }

    #[test]
    fn test_deserialize_trait() {
        struct HelloWorld {
            name: String,
            surname: String,
            age: usize,
            active: bool,
        }

        impl Deserialize for HelloWorld {
            type Error = String;

            fn from_json_object(json: &JsonValue) -> Result<Self, Self::Error> {
                let object = HelloWorld {
                    name: unwrap_option_or_err!(
                        json["name"].to_string(),
                        "field 'name' is required"
                    ),
                    surname: unwrap_option_or_err!(
                        json["surname"].to_string(),
                        "field 'surname' is required"
                    ),
                    age: unwrap_option_or_err!(json["age"].as_usize(), "field 'age' is required"),
                    active: unwrap_option_or_err!(
                        json["active"].as_bool(),
                        "field 'active' is required"
                    ),
                };

                Ok(object)
            }

            fn from_json_array(json: &JsonValue) -> Result<Vec<Self>, Self::Error> {
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

        let json_content = r#"
            {
              "name": "Onur",
              "surname": "Ozkan",
              "age": 25,
              "active": true
            }
        "#;

        let json = Json::new(json_content).parse().unwrap();
        let object = HelloWorld::from_json_object(&json).unwrap();

        assert_eq!(&object.name, "Onur");
        assert_eq!(&object.surname, "Ozkan");
        assert_eq!(object.age, 25);
        assert!(object.active);

        let json_content = r#"
            [
                {
                  "name": "Onur",
                  "surname": "Ozkan",
                  "age": 25,
                  "active": true
                },
                {
                  "name": "Onur2",
                  "surname": "Ozkan2",
                  "age": 35,
                  "active": false
                },
                {
                  "name": "Onur3",
                  "surname": "Ozkan3",
                  "age": 45,
                  "active": true
                },
                {
                  "name": "Onur4",
                  "surname": "Ozkan4",
                  "age": 55,
                  "active": false
                }
            ]
        "#;

        let json = Json::new(json_content).parse().unwrap();
        let object_array = HelloWorld::from_json_array(&json).unwrap();

        assert_eq!(&object_array[0].name, "Onur");
        assert_eq!(&object_array[0].surname, "Ozkan");
        assert_eq!(object_array[0].age, 25);
        assert!(object_array[0].active);

        assert_eq!(&object_array[1].name, "Onur2");
        assert_eq!(&object_array[1].surname, "Ozkan2");
        assert_eq!(object_array[1].age, 35);
        assert!(!object_array[1].active);

        assert_eq!(&object_array[2].name, "Onur3");
        assert_eq!(&object_array[2].surname, "Ozkan3");
        assert_eq!(object_array[2].age, 45);
        assert!(object_array[2].active);

        assert_eq!(&object_array[3].name, "Onur4");
        assert_eq!(&object_array[3].surname, "Ozkan4");
        assert_eq!(object_array[3].age, 55);
        assert!(!object_array[3].active);
    }

    #[test]
    fn test_iterator_parsing() {
        let json_content = r#"
            {
                "name": "gc-devel",
                "arch": "amd64",
                "installed_size": 3147,
                "version": {
                    "readable_format": "8.2.4",
                    "major": 8,
                    "minor": 2,
                    "patch": 4,
                    "tag": null,
                    "condition": ""
                },
                "dependencies": [
                    {
                        "name": "gc",
                        "version": {
                            "readable_format": "8.2.4",
                            "major": 8,
                            "minor": 2,
                            "patch": 4,
                            "tag": null,
                            "condition": "="
                        }
                    }
                ],
                "suggestions": []
            }
        "#;

        let mut root_object: BTreeMap<String, JsonValue> = Default::default();

        root_object.insert("name".to_string(), JsonValue::Plain("gc-devel".to_string()));
        root_object.insert("arch".to_string(), JsonValue::Plain("amd64".to_string()));
        root_object.insert(
            "installed_size".to_string(),
            JsonValue::Plain("3147".to_string()),
        );

        let mut root_version: BTreeMap<String, JsonValue> = Default::default();
        root_version.insert(
            "readable_format".to_string(),
            JsonValue::Plain("8.2.4".to_string()),
        );
        root_version.insert("condition".to_string(), JsonValue::Plain("".to_string()));
        root_version.insert("major".to_string(), JsonValue::Plain("8".to_string()));
        root_version.insert("minor".to_string(), JsonValue::Plain("2".to_string()));
        root_version.insert("patch".to_string(), JsonValue::Plain("4".to_string()));
        root_version.insert("tag".to_string(), JsonValue::Null);

        root_object.insert("version".to_string(), JsonValue::Object(root_version));

        let mut dependency_object: BTreeMap<String, JsonValue> = Default::default();
        dependency_object.insert("name".to_string(), JsonValue::Plain("gc".to_string()));

        let mut dependency_version: BTreeMap<String, JsonValue> = Default::default();
        dependency_version.insert(
            "readable_format".to_string(),
            JsonValue::Plain("8.2.4".to_string()),
        );
        dependency_version.insert("condition".to_string(), JsonValue::Plain("=".to_string()));
        dependency_version.insert("major".to_string(), JsonValue::Plain("8".to_string()));
        dependency_version.insert("minor".to_string(), JsonValue::Plain("2".to_string()));
        dependency_version.insert("patch".to_string(), JsonValue::Plain("4".to_string()));
        dependency_version.insert("tag".to_string(), JsonValue::Null);
        dependency_object.insert("version".to_string(), JsonValue::Object(dependency_version));

        root_object.insert(
            "dependencies".to_string(),
            JsonValue::Array(vec![JsonValue::Object(dependency_object)]),
        );

        root_object.insert("suggestions".to_string(), JsonValue::Array(vec![]));

        let expected = Json::new(json_content).parse().unwrap();

        assert_eq!(JsonValue::Object(root_object), expected);
    }
}
