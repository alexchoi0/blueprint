use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set, ActiveValue,
};
use uuid::Uuid;
use chrono::Utc;

use super::entities::{
    plan, op, op_result, approval,
    PlanStatus, OpStatus,
};

pub struct Repository {
    db: DatabaseConnection,
}

impl Repository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_plan(
        &self,
        name: Option<String>,
        script_path: &str,
        script_hash: &str,
    ) -> Result<plan::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let plan = plan::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name),
            script_path: Set(script_path.to_string()),
            script_hash: Set(script_hash.to_string()),
            plan_data: Set(None),
            status: Set(PlanStatus::Planning),
            created_at: Set(now),
            updated_at: Set(now),
        };
        plan.insert(&self.db).await
    }

    pub async fn save_plan_data(
        &self,
        id: Uuid,
        plan_data: Vec<u8>,
    ) -> Result<plan::Model, sea_orm::DbErr> {
        let mut plan: plan::ActiveModel = plan::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Plan not found".to_string()))?
            .into();

        plan.plan_data = Set(Some(plan_data));
        plan.updated_at = Set(Utc::now());
        plan.update(&self.db).await
    }

    pub async fn get_plan(&self, id: Uuid) -> Result<Option<plan::Model>, sea_orm::DbErr> {
        plan::Entity::find_by_id(id).one(&self.db).await
    }

    pub async fn get_plan_by_script_hash(
        &self,
        script_hash: &str,
    ) -> Result<Option<plan::Model>, sea_orm::DbErr> {
        plan::Entity::find()
            .filter(plan::Column::ScriptHash.eq(script_hash))
            .order_by_desc(plan::Column::CreatedAt)
            .one(&self.db)
            .await
    }

    pub async fn update_plan_status(
        &self,
        id: Uuid,
        status: PlanStatus,
    ) -> Result<plan::Model, sea_orm::DbErr> {
        let mut plan: plan::ActiveModel = plan::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Plan not found".to_string()))?
            .into();

        plan.status = Set(status);
        plan.updated_at = Set(Utc::now());
        plan.update(&self.db).await
    }

    pub async fn list_plans(&self) -> Result<Vec<plan::Model>, sea_orm::DbErr> {
        plan::Entity::find()
            .order_by_desc(plan::Column::CreatedAt)
            .all(&self.db)
            .await
    }

    pub async fn delete_plan(&self, id: Uuid) -> Result<(), sea_orm::DbErr> {
        plan::Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn create_op(
        &self,
        plan_id: Uuid,
        op_id: i64,
        kind: &str,
        inputs_json: &str,
        dependencies_json: Option<String>,
        level: i32,
    ) -> Result<op::Model, sea_orm::DbErr> {
        let op = op::ActiveModel {
            id: ActiveValue::NotSet,
            plan_id: Set(plan_id),
            op_id: Set(op_id),
            kind: Set(kind.to_string()),
            inputs_json: Set(inputs_json.to_string()),
            dependencies_json: Set(dependencies_json),
            level: Set(level),
            status: Set(OpStatus::Pending),
            created_at: Set(Utc::now()),
        };
        op.insert(&self.db).await
    }

    pub async fn get_ops_for_plan(&self, plan_id: Uuid) -> Result<Vec<op::Model>, sea_orm::DbErr> {
        op::Entity::find()
            .filter(op::Column::PlanId.eq(plan_id))
            .order_by_asc(op::Column::Level)
            .order_by_asc(op::Column::OpId)
            .all(&self.db)
            .await
    }

    pub async fn get_op(&self, id: i64) -> Result<Option<op::Model>, sea_orm::DbErr> {
        op::Entity::find_by_id(id).one(&self.db).await
    }

    pub async fn update_op_status(
        &self,
        id: i64,
        status: OpStatus,
    ) -> Result<op::Model, sea_orm::DbErr> {
        let mut op: op::ActiveModel = op::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Op not found".to_string()))?
            .into();

        op.status = Set(status);
        op.update(&self.db).await
    }

    pub async fn create_op_result(
        &self,
        op_id: i64,
        value_json: &str,
        input_hash: &str,
        error: Option<String>,
        duration_ms: i32,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<op_result::Model, sea_orm::DbErr> {
        let result = op_result::ActiveModel {
            id: ActiveValue::NotSet,
            op_id: Set(op_id),
            value_json: Set(value_json.to_string()),
            input_hash: Set(input_hash.to_string()),
            error: Set(error),
            duration_ms: Set(duration_ms),
            executed_at: Set(Utc::now()),
            expires_at: Set(expires_at),
        };
        result.insert(&self.db).await
    }

    pub async fn get_op_result(&self, op_id: i64) -> Result<Option<op_result::Model>, sea_orm::DbErr> {
        op_result::Entity::find()
            .filter(op_result::Column::OpId.eq(op_id))
            .one(&self.db)
            .await
    }

    pub async fn get_cached_result(
        &self,
        op_id: i64,
        input_hash: &str,
    ) -> Result<Option<op_result::Model>, sea_orm::DbErr> {
        let now = Utc::now();
        op_result::Entity::find()
            .filter(op_result::Column::OpId.eq(op_id))
            .filter(op_result::Column::InputHash.eq(input_hash))
            .filter(
                op_result::Column::ExpiresAt.is_null()
                    .or(op_result::Column::ExpiresAt.gt(now))
            )
            .one(&self.db)
            .await
    }

    pub async fn clear_cache_for_plan(&self, plan_id: Uuid) -> Result<u64, sea_orm::DbErr> {
        let ops = self.get_ops_for_plan(plan_id).await?;
        let op_ids: Vec<i64> = ops.iter().map(|o| o.id).collect();

        let result = op_result::Entity::delete_many()
            .filter(op_result::Column::OpId.is_in(op_ids))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }

    pub async fn create_approval(
        &self,
        op_id: i64,
        approved: bool,
        approved_by: Option<String>,
        resolved_value: Option<String>,
    ) -> Result<approval::Model, sea_orm::DbErr> {
        let approval = approval::ActiveModel {
            id: ActiveValue::NotSet,
            op_id: Set(op_id),
            approved: Set(approved),
            approved_by: Set(approved_by),
            approved_at: Set(Utc::now()),
            resolved_value: Set(resolved_value),
        };
        approval.insert(&self.db).await
    }

    pub async fn get_approval(&self, op_id: i64) -> Result<Option<approval::Model>, sea_orm::DbErr> {
        approval::Entity::find()
            .filter(approval::Column::OpId.eq(op_id))
            .one(&self.db)
            .await
    }
}
