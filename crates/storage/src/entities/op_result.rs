use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "op_results")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub op_id: i64,
    #[sea_orm(column_type = "Text")]
    pub value_json: String,
    pub input_hash: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub error: Option<String>,
    pub duration_ms: i32,
    pub executed_at: DateTimeUtc,
    #[sea_orm(nullable)]
    pub expires_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::op::Entity",
        from = "Column::OpId",
        to = "super::op::Column::Id"
    )]
    Op,
}

impl Related<super::op::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Op.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
