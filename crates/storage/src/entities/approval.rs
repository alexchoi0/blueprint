use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "approvals")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub op_id: i64,
    pub approved: bool,
    #[sea_orm(nullable)]
    pub approved_by: Option<String>,
    pub approved_at: DateTimeUtc,
    #[sea_orm(column_type = "Text", nullable)]
    pub resolved_value: Option<String>,
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
