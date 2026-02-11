use stencila_codemode::{Limits, LogLevel, Sandbox};

// ============================================================
// §3.3.1 / §9 — Console capture
// ============================================================

#[tokio::test]
async fn console_log_basic() {
    let sandbox = Sandbox::new(None).await.expect("sandbox");
    let response = sandbox.execute("console.log('hello world')").await;

    assert_eq!(response.logs.len(), 1);
    assert_eq!(response.logs[0].level, LogLevel::Log);
    assert_eq!(response.logs[0].message, "hello world");
}

#[tokio::test]
async fn console_all_four_levels() {
    let sandbox = Sandbox::new(None).await.expect("sandbox");
    let response = sandbox
        .execute(
            r#"
        console.debug('d');
        console.log('l');
        console.warn('w');
        console.error('e');
    "#,
        )
        .await;

    assert_eq!(response.logs.len(), 4);
    assert_eq!(response.logs[0].level, LogLevel::Debug);
    assert_eq!(response.logs[0].message, "d");
    assert_eq!(response.logs[1].level, LogLevel::Log);
    assert_eq!(response.logs[1].message, "l");
    assert_eq!(response.logs[2].level, LogLevel::Warn);
    assert_eq!(response.logs[2].message, "w");
    assert_eq!(response.logs[3].level, LogLevel::Error);
    assert_eq!(response.logs[3].message, "e");
}

#[tokio::test]
async fn console_timestamps_increase() {
    let sandbox = Sandbox::new(None).await.expect("sandbox");
    let response = sandbox
        .execute(
            r#"
        console.log('first');
        console.log('second');
        console.log('third');
    "#,
        )
        .await;

    assert_eq!(response.logs.len(), 3);
    // Timestamps should be non-decreasing
    assert!(response.logs[0].time_ms <= response.logs[1].time_ms);
    assert!(response.logs[1].time_ms <= response.logs[2].time_ms);
}

// ============================================================
// §3.3.1 — Console serialization
// ============================================================

#[tokio::test]
async fn console_primitive_serialization() {
    let sandbox = Sandbox::new(None).await.expect("sandbox");
    let response = sandbox
        .execute(
            r#"
        console.log(42);
        console.log(true);
        console.log(null);
        console.log(undefined);
        console.log('text');
    "#,
        )
        .await;

    assert_eq!(response.logs.len(), 5);
    assert_eq!(response.logs[0].message, "42");
    assert_eq!(response.logs[1].message, "true");
    assert_eq!(response.logs[2].message, "null");
    assert_eq!(response.logs[3].message, "undefined");
    assert_eq!(response.logs[4].message, "text");
}

#[tokio::test]
async fn console_object_serialization() {
    let sandbox = Sandbox::new(None).await.expect("sandbox");
    let response = sandbox.execute(r#"console.log({a: 1, b: "two"})"#).await;

    assert_eq!(response.logs.len(), 1);
    // JSON.stringify produces a stable serialization
    let parsed: serde_json::Value =
        serde_json::from_str(&response.logs[0].message).expect("valid JSON");
    assert_eq!(parsed["a"], 1);
    assert_eq!(parsed["b"], "two");
}

#[tokio::test]
async fn console_multiple_args_concatenated() {
    let sandbox = Sandbox::new(None).await.expect("sandbox");
    let response = sandbox.execute("console.log('a', 'b', 'c')").await;

    assert_eq!(response.logs.len(), 1);
    assert_eq!(response.logs[0].message, "a b c");
}

#[tokio::test]
async fn console_circular_object_fallback() {
    let sandbox = Sandbox::new(None).await.expect("sandbox");
    let response = sandbox
        .execute(
            r#"
        let obj = {};
        obj.self = obj;
        console.log(obj);
    "#,
        )
        .await;

    assert_eq!(response.logs.len(), 1);
    assert_eq!(response.logs[0].message, "[Unserializable Object]");
}

// ============================================================
// §3.3.1 — Log truncation
// ============================================================

#[tokio::test]
async fn log_truncation_at_byte_limit() {
    let limits = Limits {
        timeout_ms: None,
        max_memory_bytes: None,
        max_log_bytes: Some(20),
        max_tool_calls: None,
    };
    let sandbox = Sandbox::new(Some(&limits)).await.expect("sandbox");
    let response = sandbox
        .execute(
            r#"
        console.log('short');
        console.log('this message should be truncated because it exceeds the limit');
    "#,
        )
        .await;

    // First log captured, then truncation warning
    assert_eq!(response.logs.len(), 2);
    assert_eq!(response.logs[0].level, LogLevel::Log);
    assert_eq!(response.logs[0].message, "short");
    assert_eq!(response.logs[1].level, LogLevel::Warn);
    assert!(response.logs[1].message.contains("truncated"));
}

#[tokio::test]
async fn log_truncation_no_further_logs() {
    let limits = Limits {
        timeout_ms: None,
        max_memory_bytes: None,
        max_log_bytes: Some(5),
        max_tool_calls: None,
    };
    let sandbox = Sandbox::new(Some(&limits)).await.expect("sandbox");
    let response = sandbox
        .execute(
            r#"
        console.log('this exceeds the limit');
        console.log('this should not appear');
        console.log('nor this');
    "#,
        )
        .await;

    // Only the truncation warning should appear
    assert_eq!(response.logs.len(), 1);
    assert_eq!(response.logs[0].level, LogLevel::Warn);
    assert!(response.logs[0].message.contains("truncated"));
}
