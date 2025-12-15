use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use blueprint_common::{Plan, OpKind, OpId};
use blueprint_common::op::{ValueRef, RecordedValue};
use blueprint_interpreter::{BlueprintInterpreter, OpCache};

use starlark::environment::{GlobalsBuilder, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect};

fn bench_starlark_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("starlark_parse");

    let small_script = r#"
x = 1 + 2
y = x * 3
"#;

    let medium_script = (0..100)
        .map(|i| format!("var_{} = {} + {}", i, i, i + 1))
        .collect::<Vec<_>>()
        .join("\n");

    let large_script = (0..1000)
        .map(|i| format!("var_{} = {} + {}", i, i, i + 1))
        .collect::<Vec<_>>()
        .join("\n");

    group.bench_function("small_10_lines", |b| {
        b.iter(|| {
            black_box(AstModule::parse("bench.star", small_script.to_string(), &Dialect::Standard))
        });
    });

    group.bench_function("medium_100_lines", |b| {
        b.iter(|| {
            black_box(AstModule::parse("bench.star", medium_script.clone(), &Dialect::Standard))
        });
    });

    group.bench_function("large_1000_lines", |b| {
        b.iter(|| {
            black_box(AstModule::parse("bench.star", large_script.clone(), &Dialect::Standard))
        });
    });

    group.finish();
}

fn bench_starlark_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("starlark_eval");

    let small_script = r#"
x = 1 + 2
y = x * 3
z = y - 1
"#;

    let medium_script = (0..100)
        .map(|i| format!("var_{} = {} + {}", i, i, i + 1))
        .collect::<Vec<_>>()
        .join("\n");

    let loop_script = r#"
def run_loop():
    result = 0
    for i in range(1000):
        result = result + i
    return result

result = run_loop()
"#;

    let function_script = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

result = fib(20)
"#;

    group.bench_function("small_arithmetic", |b| {
        let ast = AstModule::parse("bench.star", small_script.to_string(), &Dialect::Standard).unwrap();
        let globals = GlobalsBuilder::standard().build();

        b.iter(|| {
            let module = Module::new();
            let mut eval = Evaluator::new(&module);
            let result = eval.eval_module(ast.clone(), &globals);
            drop(eval);
            black_box(result.is_ok())
        });
    });

    group.bench_function("medium_100_vars", |b| {
        let ast = AstModule::parse("bench.star", medium_script.clone(), &Dialect::Standard).unwrap();
        let globals = GlobalsBuilder::standard().build();

        b.iter(|| {
            let module = Module::new();
            let mut eval = Evaluator::new(&module);
            let result = eval.eval_module(ast.clone(), &globals);
            drop(eval);
            black_box(result.is_ok())
        });
    });

    group.bench_function("loop_1000_iterations", |b| {
        let ast = AstModule::parse("bench.star", loop_script.to_string(), &Dialect::Standard).unwrap();
        let globals = GlobalsBuilder::standard().build();

        b.iter(|| {
            let module = Module::new();
            let mut eval = Evaluator::new(&module);
            let result = eval.eval_module(ast.clone(), &globals);
            drop(eval);
            black_box(result.is_ok())
        });
    });

    group.bench_function("recursive_fib_20", |b| {
        let ast = AstModule::parse("bench.star", function_script.to_string(), &Dialect::Standard).unwrap();
        let globals = GlobalsBuilder::standard().build();

        b.iter(|| {
            let module = Module::new();
            let mut eval = Evaluator::new(&module);
            let result = eval.eval_module(ast.clone(), &globals);
            drop(eval);
            black_box(result.is_ok())
        });
    });

    group.finish();
}

fn bench_plan_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("plan_creation");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut plan = Plan::new();
                for i in 0..size {
                    plan.add_op(
                        OpKind::Print {
                            message: ValueRef::literal_string(format!("message {}", i)),
                        },
                        None,
                    );
                }
                black_box(plan)
            });
        });
    }

    group.finish();
}

fn bench_plan_with_dependencies(c: &mut Criterion) {
    let mut group = c.benchmark_group("plan_with_deps");

    for size in [10, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut plan = Plan::new();

                let first = plan.add_op(OpKind::Now, None);

                let mut prev = first;
                for _ in 1..size {
                    let op = plan.add_op(
                        OpKind::Print {
                            message: ValueRef::OpOutput {
                                op: prev,
                                path: vec![],
                            },
                        },
                        None,
                    );
                    prev = op;
                }

                black_box(plan)
            });
        });
    }

    group.finish();
}

fn bench_compute_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("compute_levels");

    for size in [10, 100, 500].iter() {
        let mut plan = Plan::new();

        let first = plan.add_op(OpKind::Now, None);
        let mut prev = first;
        for _ in 1..*size {
            let op = plan.add_op(
                OpKind::Print {
                    message: ValueRef::OpOutput { op: prev, path: vec![] },
                },
                None,
            );
            prev = op;
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), &plan, |b, plan| {
            b.iter(|| {
                black_box(plan.compute_levels())
            });
        });
    }

    group.finish();
}

fn bench_parallel_plan_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_plan_levels");

    for size in [10, 100, 500].iter() {
        let mut plan = Plan::new();

        for i in 0..*size {
            plan.add_op(
                OpKind::Print {
                    message: ValueRef::literal_string(format!("parallel op {}", i)),
                },
                None,
            );
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), &plan, |b, plan| {
            b.iter(|| {
                black_box(plan.compute_levels())
            });
        });
    }

    group.finish();
}

fn bench_value_ref_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_ref");

    group.bench_function("literal_string", |b| {
        b.iter(|| {
            black_box(ValueRef::literal_string("hello world"))
        });
    });

    group.bench_function("literal_int", |b| {
        b.iter(|| {
            black_box(ValueRef::literal_int(42))
        });
    });

    group.bench_function("literal_list_3", |b| {
        b.iter(|| {
            black_box(ValueRef::literal_list(vec![
                RecordedValue::Int(1),
                RecordedValue::Int(2),
                RecordedValue::Int(3),
            ]))
        });
    });

    group.bench_function("literal_list_100", |b| {
        let items: Vec<RecordedValue> = (0..100).map(|i| RecordedValue::Int(i)).collect();
        b.iter(|| {
            black_box(ValueRef::literal_list(items.clone()))
        });
    });

    group.finish();
}

fn bench_op_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("op_cache");

    group.bench_function("insert_100", |b| {
        b.iter(|| {
            let mut cache = OpCache::new();
            for i in 0..100u64 {
                cache.insert(
                    OpId(i),
                    RecordedValue::String(format!("result {}", i)),
                    i,
                );
            }
            black_box(cache)
        });
    });

    group.bench_function("get_100", |b| {
        let mut cache = OpCache::new();
        for i in 0..100u64 {
            cache.insert(
                OpId(i),
                RecordedValue::String(format!("result {}", i)),
                i,
            );
        }
        b.iter(|| {
            for i in 0..100u64 {
                black_box(cache.get(OpId(i), i));
            }
        });
    });

    group.finish();
}

fn bench_executor(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("executor");

    group.bench_function("empty_plan", |b| {
        b.to_async(&rt).iter(|| async {
            let mut executor = BlueprintInterpreter::new();
            let plan = Plan::new();
            executor.add_plan(&plan);
            black_box(executor.run().await)
        });
    });

    for size in [1, 10, 50].iter() {
        let mut plan = Plan::new();
        for _ in 0..*size {
            plan.add_op(OpKind::Now, None);
        }

        group.bench_with_input(BenchmarkId::new("now_ops", size), &plan, |b, plan| {
            b.to_async(&rt).iter(|| async {
                let mut executor = BlueprintInterpreter::new();
                executor.add_plan(plan);
                black_box(executor.run().await)
            });
        });
    }

    for size in [10, 50].iter() {
        let mut plan = Plan::new();
        for i in 0..*size {
            plan.add_op(
                OpKind::Print {
                    message: ValueRef::literal_string(format!("message {}", i)),
                },
                None,
            );
        }

        group.bench_with_input(BenchmarkId::new("print_ops", size), &plan, |b, plan| {
            b.to_async(&rt).iter(|| async {
                let mut executor = BlueprintInterpreter::new();
                executor.add_plan(plan);
                black_box(executor.run().await)
            });
        });
    }

    group.finish();
}

fn bench_validation(c: &mut Criterion) {
    use blueprint_generator::PlanValidator;

    let mut group = c.benchmark_group("validation");

    for size in [10, 100, 500].iter() {
        let mut plan = Plan::new();
        let first = plan.add_op(OpKind::Now, None);
        let mut prev = first;
        for _ in 1..*size {
            let op = plan.add_op(
                OpKind::Print {
                    message: ValueRef::OpOutput { op: prev, path: vec![] },
                },
                None,
            );
            prev = op;
        }

        group.bench_with_input(BenchmarkId::new("sequential", size), &plan, |b, plan| {
            b.iter(|| {
                black_box(PlanValidator::validate(plan, None))
            });
        });
    }

    for size in [10, 100, 500].iter() {
        let mut plan = Plan::new();
        for i in 0..*size {
            plan.add_op(
                OpKind::Print {
                    message: ValueRef::literal_string(format!("msg {}", i)),
                },
                None,
            );
        }

        group.bench_with_input(BenchmarkId::new("parallel", size), &plan, |b, plan| {
            b.iter(|| {
                black_box(PlanValidator::validate(plan, None))
            });
        });
    }

    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("full_pipeline");

    group.bench_function("parse_create_validate_10ops", |b| {
        let script = r#"
x1 = 1
x2 = 2
x3 = 3
x4 = 4
x5 = 5
x6 = 6
x7 = 7
x8 = 8
x9 = 9
x10 = 10
"#;
        b.iter(|| {
            let ast = AstModule::parse("bench.star", script.to_string(), &Dialect::Standard).unwrap();
            let globals = GlobalsBuilder::standard().build();
            let module = Module::new();
            let mut eval = Evaluator::new(&module);
            let _ = eval.eval_module(ast, &globals);

            let mut plan = Plan::new();
            for i in 0..10 {
                plan.add_op(
                    OpKind::Print {
                        message: ValueRef::literal_string(format!("x{}", i)),
                    },
                    None,
                );
            }
            let _ = plan.compute_levels();
            black_box(plan)
        });
    });

    group.bench_function("execute_10_now_ops", |b| {
        let mut plan = Plan::new();
        for _ in 0..10 {
            plan.add_op(OpKind::Now, None);
        }

        b.to_async(&rt).iter(|| {
            let plan = plan.clone();
            async move {
                let mut executor = BlueprintInterpreter::new();
                executor.add_plan(&plan);
                black_box(executor.run().await)
            }
        });
    });

    group.finish();
}

fn bench_plan_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("plan_serialization");

    for size in [10, 100, 500].iter() {
        let mut plan = Plan::new();
        let first = plan.add_op(OpKind::Now, None);
        let mut prev = first;
        for _ in 1..*size {
            let op = plan.add_op(
                OpKind::Print {
                    message: ValueRef::OpOutput { op: prev, path: vec![] },
                },
                None,
            );
            prev = op;
        }

        group.bench_with_input(BenchmarkId::new("json_serialize", size), &plan, |b, plan| {
            b.iter(|| {
                black_box(serde_json::to_string(plan).unwrap())
            });
        });

        let json = serde_json::to_string(&plan).unwrap();
        group.bench_with_input(BenchmarkId::new("json_deserialize", size), &json, |b, json| {
            b.iter(|| {
                black_box(serde_json::from_str::<Plan>(json).unwrap())
            });
        });

        group.bench_with_input(BenchmarkId::new("bincode_serialize", size), &plan, |b, plan| {
            b.iter(|| {
                black_box(bincode::serialize(plan).unwrap())
            });
        });

        let bincode_data = bincode::serialize(&plan).unwrap();
        group.bench_with_input(BenchmarkId::new("bincode_deserialize", size), &bincode_data, |b, data| {
            b.iter(|| {
                black_box(bincode::deserialize::<Plan>(data).unwrap())
            });
        });
    }

    group.finish();
}

fn bench_schema_cache(c: &mut Criterion) {
    use blueprint_generator::SchemaCache;
    use blueprint_common::Schema;

    let mut group = c.benchmark_group("schema_cache");

    let schema = Schema::new();

    group.bench_function("hash_computation", |b| {
        let content = "x = 1 + 2\ny = x * 3\nz = y - 1";
        b.iter(|| {
            black_box(SchemaCache::compute_hash(content))
        });
    });

    group.bench_function("cache_insert", |b| {
        let mut cache = SchemaCache::new();
        let mut i = 0u64;
        b.iter(|| {
            let hash = format!("hash_{}", i);
            cache.insert(hash, schema.clone());
            i += 1;
        });
    });

    group.bench_function("cache_hit", |b| {
        let mut cache = SchemaCache::new();
        for i in 0..100 {
            let hash = format!("hash_{}", i);
            cache.insert(hash, schema.clone());
        }

        b.iter(|| {
            black_box(cache.get("hash_50"))
        });
    });

    group.bench_function("cache_miss", |b| {
        let mut cache = SchemaCache::new();
        for i in 0..100 {
            let hash = format!("hash_{}", i);
            cache.insert(hash, schema.clone());
        }

        b.iter(|| {
            black_box(cache.get("nonexistent"))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_starlark_parse,
    bench_starlark_eval,
    bench_plan_creation,
    bench_plan_with_dependencies,
    bench_compute_levels,
    bench_parallel_plan_levels,
    bench_value_ref_creation,
    bench_op_cache,
    bench_executor,
    bench_validation,
    bench_full_pipeline,
    bench_plan_serialization,
    bench_schema_cache,
);

criterion_main!(benches);
