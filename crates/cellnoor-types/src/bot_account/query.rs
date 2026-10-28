use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    bot_account::BotAccount,
    operator::{BoolOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, Field, OrderField, SimpleQuery},
};

#[predicate_enum(BotAccountField)]
pub enum BotAccountPredicate {
    Id(UuidOperator),
    Description(StringOperator),
    OwnedBy(UuidOperator),
    IsStaff(BoolOperator),
    CreatedAt(TimestampOperator),
}

impl Field for BotAccountField {
    const RELATION: &'static str = <BotAccount as Relation>::NAME;
}

impl OrderField for BotAccountField {
    fn default_field() -> Self {
        Self::CreatedAt
    }
}

pub type BotAccountQuery = ComplexQuery<BotAccountPredicate, BotAccountField>;

pub type SimpleBotAccountQuery = SimpleQuery<BotAccountField>;
