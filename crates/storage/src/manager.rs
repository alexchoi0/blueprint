use sea_orm::{Database, DatabaseConnection, DbErr, Schema, ConnectionTrait};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use chrono::Utc;

use super::entities::{plan, op, op_result, approval, PlanStatus, OpStatus};
use super::repository::Repository;
use blueprint_common::{Plan, OpId, PLAN_SCHEMA_VERSION, RecordedValue};

pub struct StateManager {
    repo: Repository,
    db: DatabaseConnection,
}

impl StateManager {
    pub async fn new(database_url: &str) -> Result<Self, DbErr> {
        let db = Database::connect(database_url).await?;
        let repo = Repository::new(db.clone());
        Ok(Self { repo, db })
    }

    pub async fn new_sqlite(path: &str) -> Result<Self, DbErr> {
        let url = format!("sqlite://{}?mode=rwc", path);
        Self::new(&url).await
    }

    pub async fn new_memory() -> Result<Self, DbErr> {
        Self::new("sqlite::memory:").await
    }

    pub async fn initialize(&self) -> Result<(), DbErr> {
        let builder = self.db.get_database_backend();
        let schema = Schema::new(builder);

        let stmts = vec![
            schema.create_table_from_entity(plan::Entity),
            schema.create_table_from_entity(op::Entity),
            schema.create_table_from_entity(op_result::Entity),
            schema.create_table_from_entity(approval::Entity),
        ];

        for stmt in stmts {
            self.db.execute(builder.build(&stmt)).await?;
        }

        Ok(())
    }

    /// Compute a hash of script content.
    /// Note: This is the raw content hash without schema version.
    pub fn compute_content_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Compute a cache-safe hash that includes the schema version.
    /// This ensures cached plans are invalidated when Plan structure changes.
    pub fn compute_script_hash(content: &str) -> String {
        let content_hash = Self::compute_content_hash(content);
        format!("v{}:{}", PLAN_SCHEMA_VERSION, content_hash)
    }

    pub async fn create_or_get_plan(
        &self,
        script_path: &str,
        script_content: &str,
        name: Option<String>,
    ) -> Result<(plan::Model, bool), DbErr> {
        let script_hash = Self::compute_script_hash(script_content);

        if let Some(existing) = self.repo.get_plan_by_script_hash(&script_hash).await? {
            return Ok((existing, false));
        }

        let plan = self.repo.create_plan(name, script_path, &script_hash).await?;
        Ok((plan, true))
    }

    pub fn try_deserialize_plan(plan_model: &plan::Model) -> Option<Plan> {
        plan_model.plan_data.as_ref().and_then(|data| {
            bincode::deserialize(data).ok()
        })
    }

    pub async fn save_plan_cached(&self, plan_id: Uuid, plan: &Plan) -> Result<(), DbErr> {
        let plan_data = bincode::serialize(plan)
            .map_err(|e| DbErr::Custom(format!("Failed to serialize plan: {}", e)))?;

        self.repo.save_plan_data(plan_id, plan_data).await?;
        self.save_plan(plan_id, plan).await
    }

    pub async fn save_plan(&self, plan_id: Uuid, plan: &Plan) -> Result<(), DbErr> {
        let levels = plan.compute_levels().unwrap_or_default();

        let mut level_map = std::collections::HashMap::new();
        for (level_idx, level) in levels.iter().enumerate() {
            for op_id in level {
                level_map.insert(*op_id, level_idx as i32);
            }
        }

        for (idx, op) in plan.ops().enumerate() {
            let op_id = OpId(idx as u64);
            let level = level_map.get(&op_id).copied().unwrap_or(0);

            let kind_name = format!("{:?}", op.kind).split('{').next().unwrap_or("Unknown").trim().to_string();
            let inputs_json = serde_json::to_string(&op.kind).unwrap_or_default();

            let deps: Vec<i64> = op.inputs.iter().map(|d| d.0 as i64).collect();
            let deps_json = if deps.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&deps).unwrap_or_default())
            };

            self.repo.create_op(
                plan_id,
                idx as i64,
                &kind_name,
                &inputs_json,
                deps_json,
                level,
            ).await?;
        }

        self.repo.update_plan_status(plan_id, PlanStatus::Validated).await?;
        Ok(())
    }

    pub async fn get_plan(&self, id: Uuid) -> Result<Option<plan::Model>, DbErr> {
        self.repo.get_plan(id).await
    }

    pub async fn list_plans(&self) -> Result<Vec<plan::Model>, DbErr> {
        self.repo.list_plans().await
    }

    pub async fn delete_plan(&self, id: Uuid) -> Result<(), DbErr> {
        self.repo.delete_plan(id).await
    }

    pub async fn update_plan_status(&self, id: Uuid, status: PlanStatus) -> Result<plan::Model, DbErr> {
        self.repo.update_plan_status(id, status).await
    }

    pub async fn get_ops_for_plan(&self, plan_id: Uuid) -> Result<Vec<op::Model>, DbErr> {
        self.repo.get_ops_for_plan(plan_id).await
    }

    pub async fn update_op_status(&self, id: i64, status: OpStatus) -> Result<op::Model, DbErr> {
        self.repo.update_op_status(id, status).await
    }

    pub async fn save_op_result(
        &self,
        op_db_id: i64,
        value: &RecordedValue,
        input_hash: u64,
        duration_ms: i32,
        error: Option<String>,
    ) -> Result<op_result::Model, DbErr> {
        let value_json = serde_json::to_string(value).unwrap_or_default();
        let hash_str = format!("{:016x}", input_hash);

        self.repo.create_op_result(
            op_db_id,
            &value_json,
            &hash_str,
            error,
            duration_ms,
            None,
        ).await
    }

    pub async fn get_cached_result(
        &self,
        op_db_id: i64,
        input_hash: u64,
    ) -> Result<Option<RecordedValue>, DbErr> {
        let hash_str = format!("{:016x}", input_hash);

        if let Some(result) = self.repo.get_cached_result(op_db_id, &hash_str).await? {
            if let Ok(value) = serde_json::from_str(&result.value_json) {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    pub async fn clear_cache(&self, plan_id: Uuid) -> Result<u64, DbErr> {
        self.repo.clear_cache_for_plan(plan_id).await
    }

    pub async fn approve_op(
        &self,
        op_db_id: i64,
        approved_by: Option<String>,
    ) -> Result<approval::Model, DbErr> {
        self.repo.create_approval(op_db_id, true, approved_by, None).await
    }

    pub async fn deny_op(
        &self,
        op_db_id: i64,
        approved_by: Option<String>,
    ) -> Result<approval::Model, DbErr> {
        self.repo.create_approval(op_db_id, false, approved_by, None).await
    }

    pub async fn get_plan_summary(&self, plan_id: Uuid) -> Result<Option<PlanSummary>, DbErr> {
        let plan = match self.repo.get_plan(plan_id).await? {
            Some(p) => p,
            None => return Ok(None),
        };

        let ops = self.repo.get_ops_for_plan(plan_id).await?;

        let mut pending = 0;
        let mut completed = 0;
        let mut failed = 0;
        let total = ops.len();

        for op in &ops {
            match op.status {
                OpStatus::Pending | OpStatus::Approved => pending += 1,
                OpStatus::Completed => completed += 1,
                OpStatus::Failed => failed += 1,
                _ => {}
            }
        }

        Ok(Some(PlanSummary {
            id: plan.id,
            name: plan.name,
            script_path: plan.script_path,
            status: plan.status,
            total_ops: total,
            pending_ops: pending,
            completed_ops: completed,
            failed_ops: failed,
            created_at: plan.created_at,
            updated_at: plan.updated_at,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct PlanSummary {
    pub id: Uuid,
    pub name: Option<String>,
    pub script_path: String,
    pub status: PlanStatus,
    pub total_ops: usize,
    pub pending_ops: usize,
    pub completed_ops: usize,
    pub failed_ops: usize,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}
