//! Sorting params and responses for API endpoints.

use anyhow::{anyhow, bail, Error, Result};
use serde::de;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

/// Sort order/direction.
///
/// - `asc`: ascending order per field type (e.g. lexicographic, numeric, etc.)
/// - `desc`: descending order per field type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Asc => write!(f, "asc"),
            Self::Desc => write!(f, "desc"),
        }
    }
}

impl std::str::FromStr for SortDirection {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_ascii_lowercase();
        match s.as_str() {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            other => Err(anyhow!("invalid sort direction: {other}")),
        }
    }
}

/// Sort by a single field, e.g. `field_name:asc` or `field_name:desc`.
///
/// Defaults to ascending if direction not specified.
#[derive(
    Debug, Clone, PartialEq, Eq, serde_with::SerializeDisplay, serde_with::DeserializeFromStr,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(with = "String"))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "utoipa", schema(as = String))]
pub struct SortField {
    pub field: String,
    pub direction: SortDirection,
}

impl std::fmt::Display for SortField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.field, self.direction)
    }
}

impl std::str::FromStr for SortField {
    type Err = Error;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let raw = raw.trim();
        if raw.is_empty() {
            bail!("invalid sort field");
        }
        let (field, dir) = match raw.split_once(':') {
            Some((f, d)) => (f.trim(), Some(d.trim())),
            None => (raw, None),
        };
        if field.is_empty() {
            bail!("invalid sort field");
        }
        let direction = match dir {
            None => SortDirection::Asc,
            Some(d) => d.parse::<SortDirection>()?,
        };
        Ok(Self {
            field: field.to_owned(),
            direction,
        })
    }
}

/// Sort by multiple fields.
///
/// Results are sorted by the first field, then by the next field if the
/// first sort was a tie, and so on.
///
/// Canonical serialization is comma-separated list, e.g. `field1:asc,field2:desc`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "utoipa", schema(as = Vec<String>))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(with = "Vec<String>"))]
pub struct SortFields(pub Vec<SortField>);

impl SortFields {
    pub fn new(fields: Vec<SortField>) -> Self {
        Self(fields)
    }

    /// Get sort direction for a field.
    ///
    /// Returns the direction if the field is in the list, otherwise returns your default.
    pub fn dir(&self, field: &str, default: SortDirection) -> SortDirection {
        self.iter()
            .find(|SortField { field: f, .. }| f == field)
            .map(|SortField { direction, .. }| *direction)
            .unwrap_or(default)
    }

    /// Use defaults if no sort fields set.
    ///
    /// If the list is empty, fill it with your defaults.
    pub fn or_default(&mut self, default_sort: &[(&str, SortDirection)]) -> &mut Self {
        if self.is_empty() {
            self.0 = default_sort
                .iter()
                .map(|(f, d)| SortField {
                    field: f.to_string(),
                    direction: *d,
                })
                .collect();
        }
        self
    }

    /// Check that all fields are allowed.
    ///
    /// Returns an error if any field isn't in the allowed list.
    pub fn validate(&mut self, allowed_fields: &[&str]) -> Result<&mut Self> {
        for sf in self.iter() {
            if !allowed_fields.contains(&sf.field.as_str()) {
                bail!("invalid sort field: {}", sf.field);
            }
        }
        Ok(self)
    }

    /// Add a tie-breaker field for stable paging.
    ///
    /// Adds the field if it's not already in the list. This keeps sort order consistent
    /// across pages.
    pub fn with_tie_breaker(&mut self, field: &str, dir: SortDirection) -> &mut Self {
        if !self.iter().any(|sf| sf.field == field) {
            self.0.push(SortField {
                field: field.to_string(),
                direction: dir,
            });
        }
        self
    }
}

impl Deref for SortFields {
    type Target = [SortField];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SortFields {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum RawSortFields {
            One(String),
            Many(Vec<String>),
        }

        fn parse_one(raw: &str, out: &mut Vec<SortField>) -> Result<()> {
            if raw.trim().is_empty() {
                return Ok(());
            }
            for part in raw.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    bail!("invalid sort field");
                }
                out.push(part.parse::<SortField>()?);
            }
            Ok(())
        }

        let raw: RawSortFields = RawSortFields::deserialize(deserializer)?;
        let mut out = vec![];

        match raw {
            RawSortFields::One(s) => parse_one(&s, &mut out).map_err(de::Error::custom)?,
            RawSortFields::Many(items) => {
                for item in items {
                    parse_one(&item, &mut out).map_err(de::Error::custom)?;
                }
            }
        }

        Ok(Self(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct SortQuery {
        sort: SortFields,
    }

    #[test]
    fn deserializes_from_string_and_splits_commas() {
        let parsed: SortFields = serde_json::from_str(r#""a:asc,b:desc""#).unwrap();
        assert_eq!(
            parsed,
            SortFields::new(vec![
                SortField {
                    field: "a".to_string(),
                    direction: SortDirection::Asc,
                },
                SortField {
                    field: "b".to_string(),
                    direction: SortDirection::Desc,
                },
            ])
        );
    }

    #[test]
    fn deserializes_from_list_and_flattens_commas() {
        let parsed: SortFields = serde_json::from_str(r#"["a:asc,b:desc","c"]"#).unwrap();
        assert_eq!(
            parsed,
            SortFields::new(vec![
                SortField {
                    field: "a".to_string(),
                    direction: SortDirection::Asc,
                },
                SortField {
                    field: "b".to_string(),
                    direction: SortDirection::Desc,
                },
                SortField {
                    field: "c".to_string(),
                    direction: SortDirection::Asc,
                },
            ])
        );
    }

    #[test]
    fn invalid_direction_errors_match_existing_message() {
        let err = serde_json::from_str::<SortFields>(r#""a:sideways""#).unwrap_err();
        assert!(err.to_string().contains("invalid sort direction: sideways"));
    }

    #[test]
    fn sort_direction_parses_and_formats() {
        assert_eq!("asc".parse::<SortDirection>().unwrap(), SortDirection::Asc);
        assert_eq!("ASC".parse::<SortDirection>().unwrap(), SortDirection::Asc);
        assert_eq!(
            "desc".parse::<SortDirection>().unwrap(),
            SortDirection::Desc
        );
        assert_eq!(
            "DESC".parse::<SortDirection>().unwrap(),
            SortDirection::Desc
        );

        assert_eq!(SortDirection::Asc.to_string(), "asc");
        assert_eq!(SortDirection::Desc.to_string(), "desc");
    }

    #[test]
    fn serializes_as_list_of_strings() {
        let sort = SortFields::new(vec![
            SortField {
                field: "a".to_string(),
                direction: SortDirection::Asc,
            },
            SortField {
                field: "b".to_string(),
                direction: SortDirection::Desc,
            },
        ]);
        let json = serde_json::to_string(&sort).unwrap();
        assert_eq!(json, r#"["a:asc","b:desc"]"#);
    }

    #[test]
    fn query_params_deserialize_from_single_sort_param() {
        let parsed: SortQuery = serde_urlencoded::from_str("sort=a:asc,b:desc").unwrap();
        assert_eq!(
            parsed.sort,
            SortFields::new(vec![
                SortField {
                    field: "a".to_string(),
                    direction: SortDirection::Asc,
                },
                SortField {
                    field: "b".to_string(),
                    direction: SortDirection::Desc,
                },
            ])
        );
    }

    #[test]
    fn query_params_default_direction_is_asc() {
        let parsed: SortQuery = serde_urlencoded::from_str("sort=a").unwrap();
        assert_eq!(
            parsed.sort,
            SortFields::new(vec![SortField {
                field: "a".to_string(),
                direction: SortDirection::Asc,
            }])
        );
    }

    #[test]
    fn query_params_trims_whitespace_in_sort_string() {
        let parsed: SortQuery = serde_urlencoded::from_str("sort=a:asc,%20b:desc").unwrap();
        assert_eq!(
            parsed.sort,
            SortFields::new(vec![
                SortField {
                    field: "a".to_string(),
                    direction: SortDirection::Asc,
                },
                SortField {
                    field: "b".to_string(),
                    direction: SortDirection::Desc,
                },
            ])
        );
    }

    #[test]
    fn rejects_empty_field_name() {
        let err = ":asc".parse::<SortField>().unwrap_err();
        assert_eq!(err.to_string(), "invalid sort field");
    }

    #[test]
    fn rejects_leading_trailing_commas() {
        assert!(serde_json::from_str::<SortFields>(r#"",a:asc""#).is_err());
        assert!(serde_json::from_str::<SortFields>(r#""a:asc,""#).is_err());
        assert!(serde_json::from_str::<SortFields>(r#""a:asc,,b:desc""#).is_err());
    }

    #[test]
    fn mixed_case_direction() {
        let parsed = "name:DESC".parse::<SortField>().unwrap();
        assert_eq!(parsed.direction, SortDirection::Desc);

        let parsed = "name:Asc".parse::<SortField>().unwrap();
        assert_eq!(parsed.direction, SortDirection::Asc);
    }

    #[test]
    fn empty_string_deserializes_to_empty_sort_fields() {
        let parsed: SortFields = serde_json::from_str(r#""""#).unwrap();
        assert_eq!(parsed, SortFields::default());
    }

    #[test]
    fn repeated_query_params_are_not_supported() {
        // serde_urlencoded treats repeated keys as a duplicate field error.
        // Use comma-separated values in a single param instead: sort=a:asc,b:desc
        let result: Result<SortQuery, _> = serde_urlencoded::from_str("sort=a:asc&sort=b:desc");
        assert!(result.is_err());
    }

    #[test]
    fn dir_returns_default_when_missing() {
        let sort = SortFields::new(vec![SortField {
            field: "a".to_string(),
            direction: SortDirection::Desc,
        }]);
        assert_eq!(sort.dir("b", SortDirection::Asc), SortDirection::Asc);
        assert_eq!(sort.dir("a", SortDirection::Asc), SortDirection::Desc);
    }

    #[test]
    fn or_default_populates_when_empty() {
        let mut sort = SortFields::default();
        sort.or_default(&[("a", SortDirection::Asc)]);
        assert_eq!(
            sort,
            SortFields::new(vec![SortField {
                field: "a".to_string(),
                direction: SortDirection::Asc,
            }])
        );
    }

    #[test]
    fn or_default_noop_when_nonempty() {
        let mut sort = SortFields::new(vec![SortField {
            field: "b".to_string(),
            direction: SortDirection::Desc,
        }]);
        sort.or_default(&[("a", SortDirection::Asc)]);
        assert_eq!(
            sort,
            SortFields::new(vec![SortField {
                field: "b".to_string(),
                direction: SortDirection::Desc,
            }])
        );
    }

    #[test]
    fn validate_rejects_unknown_field() {
        let mut sort = SortFields::new(vec![SortField {
            field: "c".to_string(),
            direction: SortDirection::Asc,
        }]);
        let err = sort.validate(&["a", "b"]).unwrap_err();
        assert_eq!(err.to_string(), "invalid sort field: c");
    }

    #[test]
    fn with_tiebreaker_appends_only_if_missing() {
        let mut sort = SortFields::new(vec![SortField {
            field: "a".to_string(),
            direction: SortDirection::Desc,
        }]);
        sort.with_tie_breaker("id", SortDirection::Asc);
        assert_eq!(
            sort,
            SortFields::new(vec![
                SortField {
                    field: "a".to_string(),
                    direction: SortDirection::Desc,
                },
                SortField {
                    field: "id".to_string(),
                    direction: SortDirection::Asc,
                },
            ])
        );

        // already present — should not duplicate or change direction
        let mut sort = SortFields::new(vec![
            SortField {
                field: "a".to_string(),
                direction: SortDirection::Desc,
            },
            SortField {
                field: "id".to_string(),
                direction: SortDirection::Desc,
            },
        ]);
        sort.with_tie_breaker("id", SortDirection::Asc);
        assert_eq!(
            sort,
            SortFields::new(vec![
                SortField {
                    field: "a".to_string(),
                    direction: SortDirection::Desc,
                },
                SortField {
                    field: "id".to_string(),
                    direction: SortDirection::Desc,
                },
            ])
        );
    }

    #[test]
    fn builder_chain() {
        let mut sort = SortFields::default();
        sort.or_default(&[("a", SortDirection::Asc)])
            .validate(&["a", "id"])
            .unwrap()
            .with_tie_breaker("id", SortDirection::Asc);
        assert_eq!(
            sort,
            SortFields::new(vec![
                SortField {
                    field: "a".to_string(),
                    direction: SortDirection::Asc,
                },
                SortField {
                    field: "id".to_string(),
                    direction: SortDirection::Asc,
                },
            ])
        );
    }
}
