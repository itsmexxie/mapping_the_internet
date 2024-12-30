use std::{fmt::Display, str::FromStr};

#[cfg(feature = "diesel")]
use diesel::{
    backend::Backend, deserialize::FromSql, expression::AsExpression, serialize::ToSql, sql_types,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "diesel", derive(AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Text))]
pub enum Rir {
    Arin,
    Ripencc,
    Apnic,
    Lacnic,
    Afrinic,
    Other,
}

impl Rir {
    pub fn id(&self) -> &str {
        match self {
            Rir::Arin => "arin",
            Rir::Ripencc => "ripencc",
            Rir::Apnic => "apnic",
            Rir::Lacnic => "lacnic",
            Rir::Afrinic => "afrinic",
            Rir::Other => "other",
        }
    }
}

impl Display for Rir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Rir::Arin => "ARIN",
            Rir::Ripencc => "RIPE NCC",
            Rir::Apnic => "APNIC",
            Rir::Lacnic => "LACNIC",
            Rir::Afrinic => "AfriNIC",
            Rir::Other => "Other",
        })
    }
}

#[derive(Debug)]
pub enum RirParseErr {
    UnknownRir(String),
}

impl FromStr for Rir {
    type Err = RirParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "arin" => Ok(Rir::Arin),
            "ripencc" => Ok(Rir::Ripencc),
            "ripe ncc" => Ok(Rir::Ripencc),
            "apnic" => Ok(Rir::Apnic),
            "lacnic" => Ok(Rir::Lacnic),
            "afrinic" => Ok(Rir::Afrinic),
            _ => Err(RirParseErr::UnknownRir(s.to_string())),
        }
    }
}

#[cfg(feature = "diesel")]
impl<DB> FromSql<diesel::sql_types::Text, DB> for Rir
where
    DB: Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        match Rir::from_str(&String::from_sql(bytes)?) {
            Ok(rir) => Ok(rir),
            Err(error) => {
                Err(format!("Error while parsing RIR from database! ({:?})", error).into())
            }
        }
    }
}

#[cfg(feature = "diesel")]
impl<DB> ToSql<sql_types::Text, DB> for Rir
where
    DB: Backend,
    str: ToSql<sql_types::Text, DB>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        (self.id()).to_sql(out)
    }
}
