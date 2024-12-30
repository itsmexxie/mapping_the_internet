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
pub enum AllocationState {
    Unknown,
    Reserved,
    Unallocated,
    Allocated,
}

impl AllocationState {
    pub fn id(&self) -> &str {
        match self {
            AllocationState::Unknown => "unknown",
            AllocationState::Reserved => "reserved",
            AllocationState::Unallocated => "unallocated",
            AllocationState::Allocated => "allocated",
        }
    }
}

impl Display for AllocationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AllocationState::Unknown => "Unknown",
            AllocationState::Reserved => "Reserved",
            AllocationState::Unallocated => "Unallocated",
            AllocationState::Allocated => "Allocated",
        })
    }
}

#[derive(Debug)]
pub enum AllocStateParseErr {
    UnknownState(String),
}

impl FromStr for AllocationState {
    type Err = AllocStateParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "reserved" => Ok(AllocationState::Reserved),
            "available" => Ok(AllocationState::Unallocated),
            "allocated" => Ok(AllocationState::Allocated),
            "assigned" => Ok(AllocationState::Allocated), // Maybe not correct?
            "unknown" => Ok(AllocationState::Unknown),
            _ => Err(AllocStateParseErr::UnknownState(s.to_string())),
        }
    }
}

#[cfg(feature = "diesel")]
impl<DB> FromSql<diesel::sql_types::Text, DB> for AllocationState
where
    DB: Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        match AllocationState::from_str(&String::from_sql(bytes)?) {
            Ok(state) => Ok(state),
            Err(error) => Err(format!(
                "Error while parsing allocation state from database! ({:?})",
                error
            )
            .into()),
        }
    }
}

#[cfg(feature = "diesel")]
impl<DB> ToSql<sql_types::Text, DB> for AllocationState
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
