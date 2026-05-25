// embedded_lisp.rs
// 教育/カスタマイズ用テンプレート: Steel-coreを利用した組み込みLispエンジンの基本

use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;

// 1. Rust側からLispに提供する関数の定義
fn rust_print(text: String) {
    println!("[Rust] Lisp says: {}", text);
}

fn add_numbers(a: f64, b: f64) -> f64 {
    a + b
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 2. Steel エンジンの初期化
    let mut engine = Engine::new();

    // 3. Rust関数の登録 (マクロやトレイトを利用してラップ)
    engine.register_fn("rust:print", rust_print);
    engine.register_fn("rust:add", add_numbers);

    // 4. Lispスクリプトの評価
    let script = r#"
        ;; Rustから提供された関数を呼ぶ
        (rust:print "Hello from Steel Lisp!")

        ;; 計算を行う
        (define result (rust:add 10.5 20.0))
        (rust:print (number->string result))

        ;; 例外処理の例 (call-with-exception-handlerを使う)
        (call-with-exception-handler
            (lambda (e) (rust:print "An error occurred!"))
            (lambda () (/ 1 0)))
    "#;

    println!("Executing Lisp script...");

    // プログラムのコンパイルと実行。結果は必ずハンドリングすること。
    match engine.compile_and_run_raw_program(script) {
        Ok(results) => {
            println!("Script executed successfully. Return values: {:?}", results);
        }
        Err(e) => {
            eprintln!("Lisp Engine Error: {:?}", e);
        }
    }

    Ok(())
}
