use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum OpStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "approved")]
    Approved,
    #[sea_orm(string_value = "executing")]
    Executing,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "skipped")]
    Skipped,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ops")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub plan_id: Uuid,
    pub op_id: i64,
    pub kind: String,
    #[sea_orm(column_type = "Text")]
    pub inputs_json: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub dependencies_json: Option<String>,
    pub level: i32,
    pub status: OpStatus,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::plan::Entity",
        from = "Column::PlanId",
        to = "super::plan::Column::Id"
    )]
    Plan,
    #[sea_orm(has_one = "super::op_result::Entity")]
    OpResult,
    #[sea_orm(has_one = "super::approval::Entity")]
    Approval,
}

impl Related<super::plan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plan.def()
    }
}

impl Related<super::op_result::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OpResult.def()
    }
}

impl Related<super::approval::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Approval.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
