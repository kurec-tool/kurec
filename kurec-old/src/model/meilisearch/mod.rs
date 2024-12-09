use meilisearch_sdk::search::MatchingStrategies;
use serde::de::{self, Deserializer as SerdeDeserializer, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod program;

fn deserialize_matching_strategies<'de, D>(
    deserializer: D,
) -> Result<Option<MatchingStrategies>, D::Error>
where
    D: SerdeDeserializer<'de>,
{
    struct MatchingStrategiesVisitor;

    impl<'de> Visitor<'de> for MatchingStrategiesVisitor {
        type Value = Option<MatchingStrategies>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an optional string representing a MatchingStrategies variant")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: SerdeDeserializer<'de>,
        {
            deserializer.deserialize_str(self)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                "all" => Ok(Some(MatchingStrategies::ALL)),
                "last" => Ok(Some(MatchingStrategies::LAST)),
                "frequency" => Ok(Some(MatchingStrategies::FREQUENCY)),
                _ => Err(de::Error::unknown_variant(
                    value,
                    &["all", "last", "frequency"],
                )),
            }
        }
    }

    deserializer.deserialize_option(MatchingStrategiesVisitor)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeilisearchQuery {
    pub query: Option<String>,
    pub filter: Option<String>,
    #[serde(deserialize_with = "deserialize_matching_strategies")]
    pub matching_strategy: Option<MatchingStrategies>,
}
