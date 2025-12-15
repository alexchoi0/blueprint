use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum PlanStatus {
    #[sea_orm(string_value = "planning")]
    Planning,
    #[sea_orm(string_value = "validated")]
    Validated,
    #[sea_orm(string_value = "approved")]
    Approved,
    #[sea_orm(string_value = "executing")]
    Executing,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plans")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: Option<String>,
    pub script_path: String,
    pub script_hash: String,
    #[sea_orm(column_type = "Blob", nullable)]
    pub plan_data: Option<Vec<u8>>,
    pub status: PlanStatus,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::op::Entity")]
    Ops,
}

impl Related<super::op::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ops.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
