use crate::{iterator::iterate_tokens, json_value::JsonValue};
use std::fmt::Display;

pub struct Json<'a>(&'a str);

impl Display for Json<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Json({})", self.0)
    }
}

impl std::fmt::Debug for Json<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Json({})", self.0)
    }
}

impl<'a> Json<'a> {
    #[inline(always)]
    pub fn new(json_string: &'a str) -> Self {
        Self(json_string)
    }

    pub fn parse(&self) -> Result<JsonValue, String> {
        let mut chars: Vec<char> = self.0.chars().collect();

        let mut root_object = JsonValue::default();
        iterate_tokens(&mut chars, &mut root_object, None, None)?;

        Ok(root_object)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_json_value_index_access() {
        let json_content = r#"{
            "president": [
                {
                    "let": 1974033217,
                    "nine": 1043848105,
                    "differ": false,
                    "animal": 1083488261,
                    "section": "pale",
                    "table": "fox"
                },
                false,
                "movie",
                "bee",
                true,
                -58333673.273878574
            ],
            "thought": true,
            "with": true,
            "village": "include"
        }"#;

        let mut president = vec![];

        let mut president_inner_object = BTreeMap::new();
        president_inner_object.insert(
            "let".to_string(),
            JsonValue::Plain("1974033217".to_string()),
        );
        president_inner_object.insert(
            "nine".to_string(),
            JsonValue::Plain("1043848105".to_string()),
        );
        president_inner_object.insert("differ".to_string(), JsonValue::Plain("false".to_string()));
        president_inner_object.insert(
            "animal".to_string(),
            JsonValue::Plain("1083488261".to_string()),
        );
        president_inner_object.insert("section".to_string(), JsonValue::Plain("pale".to_string()));
        president_inner_object.insert("table".to_string(), JsonValue::Plain("fox".to_string()));
        president.push(JsonValue::Object(president_inner_object.clone()));

        president.push(JsonValue::Plain("false".to_string()));
        president.push(JsonValue::Plain("movie".to_string()));
        president.push(JsonValue::Plain("bee".to_string()));
        president.push(JsonValue::Plain("true".to_string()));
        president.push(JsonValue::Plain("-58333673.273878574".to_string()));

        let json = Json::new(&json_content);
        let json_value = json.parse().unwrap();

        assert_eq!(json_value["president"], JsonValue::Array(president));
        assert_eq!(
            json_value["president"][0],
            JsonValue::Object(president_inner_object)
        );
        assert_eq!(
            json_value["president"][0]["let"],
            JsonValue::Plain("1974033217".to_string())
        );
        assert_eq!(
            json_value["president"][0]["nine"],
            JsonValue::Plain("1043848105".to_string())
        );
        assert_eq!(
            json_value["president"][0]["differ"],
            JsonValue::Plain("false".to_string())
        );
        assert_eq!(
            json_value["president"][0]["animal"],
            JsonValue::Plain("1083488261".to_string())
        );
        assert_eq!(
            json_value["president"][0]["section"],
            JsonValue::Plain("pale".to_string())
        );
        assert_eq!(
            json_value["president"][0]["table"],
            JsonValue::Plain("fox".to_string())
        );
        assert_eq!(
            json_value["president"][1],
            JsonValue::Plain("false".to_string())
        );
        assert_eq!(
            json_value["president"][2],
            JsonValue::Plain("movie".to_string())
        );
        assert_eq!(
            json_value["president"][3],
            JsonValue::Plain("bee".to_string())
        );
        assert_eq!(
            json_value["president"][4],
            JsonValue::Plain("true".to_string())
        );
        assert_eq!(
            json_value["president"][5],
            JsonValue::Plain("-58333673.273878574".to_string())
        );
        assert_eq!(json_value["thought"], JsonValue::Plain("true".to_string()));
        assert_eq!(json_value["with"], JsonValue::Plain("true".to_string()));
        assert_eq!(
            json_value["village"],
            JsonValue::Plain("include".to_string())
        );

        assert_eq!(json_value["presssident"], JsonValue::Null);
        assert_eq!(json_value["president"][0]["x"], JsonValue::Null);
        assert_eq!(json_value["president"][1]["x"], JsonValue::Null);
        assert_eq!(json_value["president"][6], JsonValue::Null);
    }

    #[test]
    fn test_deserialization_on_json_samples() {
        // Object sample
        {
            let json_content = r#"{
                "name": "Onur",
                "surname": "Ozkan",
                "gender": "male",
                "website": "https://onurozkan.dev",
                "private_account": true,
                "frozen_account": false,
                "other_accounts": {
                    "twitter": "onurozkan_dev",
                    "linkedin": "onurozkandev",
                    "github": "ozkanonur"
                },
                "pets": [
                    {
                        "name": "boncuk",
                        "species": "cat",
                        "fav_foods": ["corn", "polenta"]
                    },
                    {
                        "name": "paskal",
                        "species": "dog",
                        "fav_foods": ["meat", "chicken", "beef"]
                    },
                    {
                        "name": "cesur",
                        "species": "dog",
                        "fav_foods": ["meat", "cheese", "chicken", "beef"]
                    }
                ]
            }"#;

            let mut root_level = BTreeMap::new();

            let mut other_accounts = BTreeMap::new();
            other_accounts.insert(
                "twitter".to_string(),
                JsonValue::Plain("onurozkan_dev".to_string()),
            );
            other_accounts.insert(
                "linkedin".to_string(),
                JsonValue::Plain("onurozkandev".to_string()),
            );
            other_accounts.insert(
                "github".to_string(),
                JsonValue::Plain("ozkanonur".to_string()),
            );

            let mut pets = vec![];

            let mut pet = BTreeMap::new();
            let mut fav_foods = vec![];
            fav_foods.push(JsonValue::Plain("corn".to_string()));
            fav_foods.push(JsonValue::Plain("polenta".to_string()));

            pet.insert("name".to_string(), JsonValue::Plain("boncuk".to_string()));
            pet.insert("species".to_string(), JsonValue::Plain("cat".to_string()));
            pet.insert("fav_foods".to_string(), JsonValue::Array(fav_foods));

            pets.push(JsonValue::Object(pet));

            let mut pet = BTreeMap::new();
            let mut fav_foods = vec![];
            fav_foods.push(JsonValue::Plain("meat".to_string()));
            fav_foods.push(JsonValue::Plain("chicken".to_string()));
            fav_foods.push(JsonValue::Plain("beef".to_string()));

            pet.insert("name".to_string(), JsonValue::Plain("paskal".to_string()));
            pet.insert("species".to_string(), JsonValue::Plain("dog".to_string()));
            pet.insert("fav_foods".to_string(), JsonValue::Array(fav_foods));

            pets.push(JsonValue::Object(pet));

            let mut pet = BTreeMap::new();
            let mut fav_foods = vec![];
            fav_foods.push(JsonValue::Plain("meat".to_string()));
            fav_foods.push(JsonValue::Plain("cheese".to_string()));
            fav_foods.push(JsonValue::Plain("chicken".to_string()));
            fav_foods.push(JsonValue::Plain("beef".to_string()));

            pet.insert("name".to_string(), JsonValue::Plain("cesur".to_string()));
            pet.insert("species".to_string(), JsonValue::Plain("dog".to_string()));
            pet.insert("fav_foods".to_string(), JsonValue::Array(fav_foods));

            pets.push(JsonValue::Object(pet));

            root_level.insert("name".to_string(), JsonValue::Plain("Onur".to_string()));
            root_level.insert("surname".to_string(), JsonValue::Plain("Ozkan".to_string()));
            root_level.insert("gender".to_string(), JsonValue::Plain("male".to_string()));
            root_level.insert(
                "website".to_string(),
                JsonValue::Plain("https://onurozkan.dev".to_string()),
            );
            root_level.insert(
                "private_account".to_string(),
                JsonValue::Plain("true".to_string()),
            );
            root_level.insert(
                "frozen_account".to_string(),
                JsonValue::Plain("false".to_string()),
            );
            root_level.insert(
                "other_accounts".to_string(),
                JsonValue::Object(other_accounts),
            );

            root_level.insert("pets".to_string(), JsonValue::Array(pets));

            let expected_object = JsonValue::Object(root_level);

            let json = Json::new(&json_content);
            let actual_object = json.parse().unwrap();

            assert_eq!(expected_object, actual_object);
        }

        // Array sample
        {
            let json_content = r#"
            [
                {
                    "id": "feaaf6b1-c637-4243-9570-c97fbaa6620c",
                    "first_name": "Hannie",
                    "last_name": "Bazire",
                    "email": "hbazire0@squarespace.com",
                    "gender": "Female",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "26.229.254.252",
                    "birth": {
                        "day": 01,
                        "month": 05,
                        "year": 1985
                    }
                },
                {
                    "id": "395605bb-bc3b-4bec-9bd2-82ab7d84825e",
                    "first_name": "Cornela",
                    "last_name": "Beckitt",
                    "email": "cbeckitt1@walmart.com",
                    "gender": "Female",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "236.125.222.140",
                    "birth": {
                        "day": 06,
                        "month": 11,
                        "year": 1995
                    }
                },
                {
                    "id": "0fa010a6-9c1d-487b-a69d-767775ae333f",
                    "first_name": "Dougy",
                    "last_name": "Filipowicz",
                    "email": "dfilipowicz2@cbslocal.com",
                    "gender": "Genderqueer",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "207.125.229.204",
                    "birth": {
                        "day": 23,
                        "month": 12,
                        "year": 1969
                    }
                },
                {
                    "id": "2a94416e-39a9-4322-b3d5-c4482e29b36f",
                    "first_name": "Brose",
                    "last_name": "Machel",
                    "email": "bmachel3@ftc.gov",
                    "gender": "Male",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "31.222.19.35",
                    "birth": {
                        "day": 14,
                        "month": 04,
                        "year": 1991
                    }
                },
                {
                    "id": "67a650cf-dabb-4f87-9c70-5719f402cd98",
                    "first_name": "Beverly",
                    "last_name": "Maudling",
                    "email": "bmaudling4@hostgator.com",
                    "gender": "Female",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "128.203.51.91",
                    "birth": {
                        "day": 07,
                        "month": 06,
                        "year": 1988
                    }
                },
                {
                    "id": "2479dbc5-fc0f-4623-9880-f80a52a60e3a",
                    "first_name": "Wait",
                    "last_name": "Behan",
                    "email": "wbehan5@smh.com.au",
                    "gender": "Male",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "217.241.113.180",
                    "birth": {
                        "day": 12,
                        "month": 02,
                        "year": 2001
                    }
                },
                {
                    "id": "77e5906b-9273-42ed-89b4-66b6fc96257b",
                    "first_name": "Regen",
                    "last_name": "de Mullett",
                    "email": "rdemullett6@aboutads.info",
                    "gender": "Male",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "118.165.112.11",
                    "birth": {
                        "day": 22,
                        "month": 09,
                        "year": 1993
                    }
                }
            ]"#;

            let mut root_level = vec![];

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("feaaf6b1-c637-4243-9570-c97fbaa6620c".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Hannie".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Bazire".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("hbazire0@squarespace.com".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Female".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("26.229.254.252".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("01".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("05".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1985".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("395605bb-bc3b-4bec-9bd2-82ab7d84825e".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Cornela".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Beckitt".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("cbeckitt1@walmart.com".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Female".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("236.125.222.140".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("06".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("11".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1995".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("0fa010a6-9c1d-487b-a69d-767775ae333f".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Dougy".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Filipowicz".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("dfilipowicz2@cbslocal.com".to_string()),
                );
                account.insert(
                    "gender".to_string(),
                    JsonValue::Plain("Genderqueer".to_string()),
                );
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("207.125.229.204".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("23".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("12".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1969".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("2a94416e-39a9-4322-b3d5-c4482e29b36f".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Brose".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Machel".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("bmachel3@ftc.gov".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Male".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("31.222.19.35".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("14".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("04".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1991".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("67a650cf-dabb-4f87-9c70-5719f402cd98".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Beverly".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Maudling".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("bmaudling4@hostgator.com".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Female".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("128.203.51.91".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("07".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("06".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1988".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("2479dbc5-fc0f-4623-9880-f80a52a60e3a".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Wait".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Behan".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("wbehan5@smh.com.au".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Male".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("217.241.113.180".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("12".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("02".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("2001".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("77e5906b-9273-42ed-89b4-66b6fc96257b".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Regen".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("de Mullett".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("rdemullett6@aboutads.info".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Male".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("118.165.112.11".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("22".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("09".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1993".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            let expected_object = JsonValue::Array(root_level);

            let json = Json::new(&json_content);
            let actual_object = json.parse().unwrap();

            assert_eq!(expected_object, actual_object);
        }

        // Another object(some more complex) sample
        {
            let json_content = r#"{
                "president": [
                    {
                        "let": 1974033217,
                        "nine": 1043848105,
                        "differ": false,
                        "animal": 1083488261,
                        "section": "pale",
                        "table": "fox"
                    },
                    false,
                    "movie",
                    "bee",
                    true,
                    -58333673.273878574
                ],
                "thought": true,
                "with": true,
                "village": "include"
            }"#;

            let mut root_level = BTreeMap::new();

            let mut president = vec![];

            let mut president_inner_object = BTreeMap::new();
            president_inner_object.insert(
                "let".to_string(),
                JsonValue::Plain("1974033217".to_string()),
            );
            president_inner_object.insert(
                "nine".to_string(),
                JsonValue::Plain("1043848105".to_string()),
            );
            president_inner_object
                .insert("differ".to_string(), JsonValue::Plain("false".to_string()));
            president_inner_object.insert(
                "animal".to_string(),
                JsonValue::Plain("1083488261".to_string()),
            );
            president_inner_object
                .insert("section".to_string(), JsonValue::Plain("pale".to_string()));
            president_inner_object.insert("table".to_string(), JsonValue::Plain("fox".to_string()));
            president.push(JsonValue::Object(president_inner_object));

            president.push(JsonValue::Plain("false".to_string()));
            president.push(JsonValue::Plain("movie".to_string()));
            president.push(JsonValue::Plain("bee".to_string()));
            president.push(JsonValue::Plain("true".to_string()));
            president.push(JsonValue::Plain("-58333673.273878574".to_string()));

            root_level.insert("president".to_string(), JsonValue::Array(president));
            root_level.insert("thought".to_string(), JsonValue::Plain("true".to_string()));
            root_level.insert("with".to_string(), JsonValue::Plain("true".to_string()));
            root_level.insert(
                "village".to_string(),
                JsonValue::Plain("include".to_string()),
            );

            let expected_object = JsonValue::Object(root_level);

            let json = Json::new(&json_content);
            let actual_object = json.parse().unwrap();

            assert_eq!(expected_object, actual_object);
        }
    }
}
