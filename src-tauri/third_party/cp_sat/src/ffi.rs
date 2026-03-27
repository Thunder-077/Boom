use crate::proto;
use prost::Message;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
const CP_MODEL_PROTO_SOURCE: &str = include_str!("cp_model.proto");

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
    let binary_output_path = temp_root.join(format!("response-{token}.bin"));
    let text_output_path = temp_root.join(format!("response-{token}.txt"));
    let runtime_proto_path = temp_root.join(format!("cp_model-{token}.proto"));

    if let Err(error) = fs::write(&input_path, &model_buf) {
        return error_response(&format!("无法写入 CP-SAT 输入模型: {error}"));
    }

    let mut preferred_text_error: Option<String> = None;
    if cfg!(windows) {
        if let Some(protoc_path) = resolve_protoc_path(&runner_path) {
            match recover_response_via_text_output(
                &runner_path,
                &protoc_path,
                &input_path,
                &text_output_path,
                &runtime_proto_path,
                params,
            ) {
                Ok(response) => {
                    cleanup_files(&[
                        &input_path,
                        &binary_output_path,
                        &text_output_path,
                        &runtime_proto_path,
                    ]);
                    return response;
                }
                Err(text_error) => {
                    preferred_text_error = Some(text_error);
                }
            }
        }
    }

    let binary_output = match run_sat_runner(&runner_path, &input_path, &binary_output_path, params) {
        Ok(output) => output,
        Err(error) => {
            cleanup_files(&[
                &input_path,
                &binary_output_path,
                &text_output_path,
                &runtime_proto_path,
            ]);
            return error_response(&format!(
                "无法启动 CP-SAT 求解器 {}: {error} (input={}, preferred_text={})",
                runner_path.display(),
                input_path.display(),
                preferred_text_error.unwrap_or_else(|| "not_attempted".to_string())
            ));
        }
    };

    if let Ok(response_bytes) = fs::read(&binary_output_path) {
        if let Ok(response) = proto::CpSolverResponse::decode(response_bytes.as_slice()) {
            cleanup_files(&[
                &input_path,
                &binary_output_path,
                &text_output_path,
                &runtime_proto_path,
            ]);
            return response;
        }
    }

    let binary_diagnostic = build_binary_output_file_diagnostic(&binary_output_path);
    if let Some(protoc_path) = resolve_protoc_path(&runner_path) {
        match recover_response_via_text_output(
            &runner_path,
            &protoc_path,
            &input_path,
            &text_output_path,
            &runtime_proto_path,
            params,
        ) {
            Ok(mut response) => {
                annotate_recovered_response(
                    &mut response,
                    binary_output.status.code(),
                    &binary_output_path,
                    &binary_diagnostic,
                );
                if !keep_failed_artifacts() {
                    cleanup_files(&[
                        &input_path,
                        &binary_output_path,
                        &text_output_path,
                        &runtime_proto_path,
                    ]);
                }
                return response;
            }
            Err(text_error) => {
                if !keep_failed_artifacts() {
                    cleanup_files(&[
                        &input_path,
                        &binary_output_path,
                        &text_output_path,
                        &runtime_proto_path,
                    ]);
                }
                return error_response(&format!(
                    "CP-SAT 二进制响应损坏，文本回退也失败: exit={:?}, stdout={}, stderr={}, input={}, output={}, file={}, preferred_text={}, text_fallback={}",
                    binary_output.status.code(),
                    String::from_utf8_lossy(&binary_output.stdout).trim(),
                    String::from_utf8_lossy(&binary_output.stderr).trim(),
                    input_path.display(),
                    binary_output_path.display(),
                    binary_diagnostic,
                    preferred_text_error
                        .as_deref()
                        .unwrap_or("not_attempted"),
                    text_error
                ));
            }
        }
    }

    if !keep_failed_artifacts() {
        cleanup_files(&[
            &input_path,
            &binary_output_path,
            &text_output_path,
            &runtime_proto_path,
        ]);
    }
    error_response(&format!(
        "CP-SAT 求解失败，且无法找到 protoc 文本回退工具: exit={:?}, stdout={}, stderr={}, input={}, output={}, file={}, protoc=missing",
        binary_output.status.code(),
        String::from_utf8_lossy(&binary_output.stdout).trim(),
        String::from_utf8_lossy(&binary_output.stderr).trim(),
        input_path.display(),
        binary_output_path.display(),
        binary_diagnostic
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

fn run_sat_runner(
    runner_path: &Path,
    input_path: &Path,
    output_path: &Path,
    params: Option<&proto::SatParameters>,
) -> Result<Output, std::io::Error> {
    let mut command = Command::new(runner_path);
    command.arg(format!("--input={}", input_path.display()));
    command.arg(format!("--output={}", output_path.display()));
    if let Some(text) = sat_parameters_to_text(params) {
        command.arg(format!("--params={text}"));
    }
    command.output()
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

fn resolve_protoc_path(runner_path: &Path) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CP_SAT_PROTOC") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    let sibling = runner_path.parent()?.join(exe_name("protoc"));
    if sibling.is_file() {
        Some(sibling)
    } else {
        None
    }
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

fn build_binary_output_file_diagnostic(path: &Path) -> String {
    let Ok(bytes) = fs::read(path) else {
        return "missing".to_string();
    };
    let decode_error = proto::CpSolverResponse::decode(bytes.as_slice())
        .err()
        .map(|error| error.to_string())
        .unwrap_or_else(|| "none".to_string());
    let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(240)])
        .replace('\r', " ")
        .replace('\n', " | ");
    format!(
        "exists,size={},decode_error={},preview={}",
        bytes.len(),
        decode_error,
        preview
    )
}

fn build_text_output_file_diagnostic(path: &Path) -> String {
    let Ok(text) = fs::read_to_string(path) else {
        return "missing".to_string();
    };
    let preview = text
        .lines()
        .take(16)
        .collect::<Vec<_>>()
        .join(" | ")
        .replace('\r', " ");
    format!("exists,chars={},preview={preview}", text.len())
}

fn recover_response_via_text_output(
    runner_path: &Path,
    protoc_path: &Path,
    input_path: &Path,
    text_output_path: &Path,
    runtime_proto_path: &Path,
    params: Option<&proto::SatParameters>,
) -> Result<proto::CpSolverResponse, String> {
    let text_run = run_sat_runner(runner_path, input_path, text_output_path, params)
        .map_err(|error| format!("无法启动文本回退 CP-SAT: {error}"))?;
    let text_content = fs::read_to_string(text_output_path).map_err(|error| {
        format!(
            "无法读取文本回退输出 {}: {error}; file={}",
            text_output_path.display(),
            build_text_output_file_diagnostic(text_output_path)
        )
    })?;
    fs::write(runtime_proto_path, CP_MODEL_PROTO_SOURCE).map_err(|error| {
        format!(
            "无法写入运行时 cp_model.proto {}: {error}",
            runtime_proto_path.display()
        )
    })?;
    let encoded = encode_text_response_with_protoc(protoc_path, runtime_proto_path, &text_content)
        .map_err(|error| {
            format!(
                "无法将文本响应重新编码为 protobuf: {}; text_exit={:?}, text_stdout={}, text_stderr={}, text_file={}",
                error,
                text_run.status.code(),
                String::from_utf8_lossy(&text_run.stdout).trim(),
                String::from_utf8_lossy(&text_run.stderr).trim(),
                build_text_output_file_diagnostic(text_output_path)
            )
        })?;
    proto::CpSolverResponse::decode(encoded.as_slice()).map_err(|error| {
        format!(
            "文本回退重新编码后仍无法解码: {error}; text_exit={:?}, text_file={}",
            text_run.status.code(),
            build_text_output_file_diagnostic(text_output_path)
        )
    })
}

fn encode_text_response_with_protoc(
    protoc_path: &Path,
    runtime_proto_path: &Path,
    text_content: &str,
) -> Result<Vec<u8>, String> {
    let proto_dir = runtime_proto_path
        .parent()
        .ok_or_else(|| "运行时 proto 目录不存在".to_string())?;
    let proto_name = runtime_proto_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "运行时 proto 文件名无效".to_string())?;

    let mut child = Command::new(protoc_path)
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg("--encode=operations_research.sat.CpSolverResponse")
        .arg(proto_name)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("无法启动 protoc {}: {error}", protoc_path.display()))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "protoc stdin 不可用".to_string())?;
        stdin
            .write_all(text_content.as_bytes())
            .map_err(|error| format!("写入 protoc stdin 失败: {error}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("等待 protoc 结束失败: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "exit={:?}, stdout={}, stderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

fn annotate_recovered_response(
    response: &mut proto::CpSolverResponse,
    binary_exit_code: Option<i32>,
    binary_output_path: &Path,
    binary_diagnostic: &str,
) {
    let note = format!(
        "[cp_sat_bridge] 已通过文本回退恢复损坏的二进制响应: exit={binary_exit_code:?}, output={}, file={binary_diagnostic}",
        binary_output_path.display()
    );
    if response.solve_log.trim().is_empty() {
        response.solve_log = note;
    } else {
        response.solve_log.push('\n');
        response.solve_log.push_str(&note);
    }
}

fn keep_failed_artifacts() -> bool {
    matches!(
        std::env::var("CP_SAT_KEEP_FAILED_ARTIFACTS").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
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
