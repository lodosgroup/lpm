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
}
