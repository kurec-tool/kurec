use shared_macros::stream_worker;

// 入力パラメータがない関数（エラーになるべき）
#[stream_worker]
async fn invalid_function_no_params() -> Result<(), anyhow::Error> {
    Ok(())
}

// 複数の入力パラメータを持つ関数（エラーになるべき）
#[stream_worker]
async fn invalid_function_multiple_params(
    param1: String,
    param2: String,
) -> Result<(), anyhow::Error> {
    Ok(())
}

// Result型を返さない関数（エラーになるべき）
#[stream_worker]
async fn invalid_function_not_result(param: String) -> String {
    param
}

fn main() {}
