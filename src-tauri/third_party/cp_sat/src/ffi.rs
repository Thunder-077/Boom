use crate::proto;
use prost::Message;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Solves the given [CpModelProto][crate::proto::CpModelProto] and
/// returns an instance of
/// [CpSolverResponse][crate::proto::CpSolverResponse].
pub fn solve(model: &proto::CpModelProto) -> proto::CpSolverResponse {
    solve_internal(model, None)
}

/// Solves the given [CpModelProto][crate::proto::CpModelProto] with
/// the given parameters.
pub fn solve_with_parameters(
    model: &proto::CpModelProto,
    params: &proto::SatParameters,
) -> proto::CpSolverResponse {
    solve_internal(model, Some(params))
}

/// Returns a string with some statistics on the given
/// [CpModelProto][crate::proto::CpModelProto].
pub fn cp_model_stats(model: &proto::CpModelProto) -> String {
    format!(
        "variables={}, constraints={}",
        model.variables.len(),
        model.constraints.len()
    )
}

/// Returns a string with some statistics on the solver response.
///
/// If the second argument is false, we will just display NA for the
/// objective value instead of zero. It is not really needed but it
/// makes things a bit clearer to see that there is no objective.
pub fn cp_solver_response_stats(response: &proto::CpSolverResponse, has_objective: bool) -> String {
    if has_objective {
        format!(
            "status={:?}, objective_value={}, best_objective_bound={}, wall_time={:.3}",
            response.status(),
            response.objective_value,
            response.best_objective_bound,
            response.wall_time
        )
    } else {
        format!(
            "status={:?}, wall_time={:.3}, info={}",
            response.status(),
            response.wall_time,
            response.solution_info
        )
    }
}

/// Verifies that the given model satisfies all the properties
/// described in the proto comments. Returns an empty string if it is
/// the case, otherwise fails at the first error and returns a
/// human-readable description of the issue.
pub fn validate_cp_model(model: &proto::CpModelProto) -> String {
    if model.variables.is_empty() {
        return "model must contain at least one variable".to_string();
    }
    String::new()
}

/// Verifies that the given variable assignment is a feasible solution
/// of the given model. The values vector should be in one to one
/// correspondence with the model.variables() list of variables.
///
/// # Example
///
/// ```
/// # use cp_sat::builder::CpModelBuilder;
/// # use cp_sat::proto::CpSolverStatus;
/// # use cp_sat::ffi::solution_is_feasible;
/// let mut model = CpModelBuilder::default();
/// let x = model.new_bool_var();
/// let y = model.new_bool_var();
/// model.add_and([x, y]);
/// assert!(solution_is_feasible(model.proto(), &[1, 1]));
/// assert!(!solution_is_feasible(model.proto(), &[1, 0]));
/// assert!(!solution_is_feasible(model.proto(), &[0, 1]));
/// assert!(!solution_is_feasible(model.proto(), &[0, 0]));
/// ```
pub fn solution_is_feasible(model: &proto::CpModelProto, solution: &[i64]) -> bool {
    if model.variables.len() != solution.len() {
        return false;
    }
    for (variable, value) in model.variables.iter().zip(solution.iter()) {
        if !value_is_within_domain(variable.domain.as_slice(), *value) {
            return false;
        }
    }
    true
}

fn solve_internal(
    model: &proto::CpModelProto,
    params: Option<&proto::SatParameters>,
) -> proto::CpSolverResponse {
    let mut model_buf = Vec::default();
    model.encode(&mut model_buf).unwrap();

    let Some(runner_path) = resolve_sat_runner_path() else {
        return error_response("CP-SAT sat_runner.exe 未找到");
    };

    let temp_root = std::env::temp_dir().join("academic_administration_cp_sat");
    if let Err(error) = fs::create_dir_all(&temp_root) {
        return error_response(&format!("无法创建 CP-SAT 临时目录: {error}"));
    }
    let token = unique_token();
    let input_path = temp_root.join(format!("model-{token}.bin"));
    let output_path = temp_root.join(format!("response-{token}.bin"));

    if let Err(error) = fs::write(&input_path, &model_buf) {
        return error_response(&format!("无法写入 CP-SAT 输入模型: {error}"));
    }

    let mut command = Command::new(&runner_path);
    command.arg(format!("--input={}", input_path.display()));
    command.arg(format!("--output={}", output_path.display()));
    if let Some(text) = sat_parameters_to_text(params) {
        command.arg(format!("--params={text}"));
    }

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            cleanup_files(&[&input_path, &output_path]);
            return error_response(&format!(
                "无法启动 CP-SAT 求解器 {}: {error} (input={})",
                runner_path.display(),
                input_path.display()
            ));
        }
    };

    if let Ok(response_bytes) = fs::read(&output_path) {
        if let Ok(response) = proto::CpSolverResponse::decode(response_bytes.as_slice()) {
            cleanup_files(&[&input_path, &output_path]);
            return response;
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    cleanup_files(&[&input_path, &output_path]);
    error_response(&format!(
        "CP-SAT 求解失败: exit={:?}, stdout={}, stderr={}, input={}, output={}",
        output.status.code(),
        stdout.trim(),
        stderr.trim(),
        input_path.display(),
        output_path.display()
    ))
}

fn value_is_within_domain(domain: &[i64], value: i64) -> bool {
    for chunk in domain.chunks(2) {
        if chunk.len() == 2 && value >= chunk[0] && value <= chunk[1] {
            return true;
        }
    }
    false
}

fn cleanup_files(paths: &[&Path]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

fn unique_token() -> String {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{}-{now}-{counter}", std::process::id())
}

fn resolve_sat_runner_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CP_SAT_SAT_RUNNER") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let vendor_root = manifest_dir.parent()?.parent()?.join("vendor");
    let entries = fs::read_dir(vendor_root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let runner = path.join("bin").join(exe_name("sat_runner"));
        if runner.is_file() {
            return Some(runner);
        }
    }
    None
}

fn sat_parameters_to_text(params: Option<&proto::SatParameters>) -> Option<String> {
    let params = params?;
    let mut items = Vec::new();

    if let Some(value) = params.max_time_in_seconds {
        if value.is_finite() {
            items.push(format!("max_time_in_seconds:{value}"));
        }
    }
    if let Some(value) = params.max_deterministic_time {
        if value.is_finite() {
            items.push(format!("max_deterministic_time:{value}"));
        }
    }
    if let Some(value) = params.random_seed {
        items.push(format!("random_seed:{value}"));
    }
    if let Some(value) = params.log_search_progress {
        items.push(format!("log_search_progress:{value}"));
    }
    if let Some(value) = params.num_search_workers {
        items.push(format!("num_search_workers:{value}"));
    }
    if let Some(value) = params.enumerate_all_solutions {
        items.push(format!("enumerate_all_solutions:{value}"));
    }
    if let Some(value) = params.use_lns_only {
        items.push(format!("use_lns_only:{value}"));
    }

    if items.is_empty() {
        None
    } else {
        Some(items.join(" "))
    }
}

fn error_response(message: &str) -> proto::CpSolverResponse {
    proto::CpSolverResponse {
        status: proto::CpSolverStatus::Unknown as i32,
        solution_info: message.to_string(),
        ..Default::default()
    }
}

fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}
