// cpp_ffi.rs
// 教育/カスタマイズ用テンプレート: RustからC++関数を呼び出す(FFI)
// ※注意: このファイルは単体ではビルドに失敗します。
// 実際のプロジェクトでは build.rs でC++コードをコンパイルし、リンク設定を行う必要があります。
// 以下のコードはビルドエラーを避けるためにコメントアウトしています。
// リンク設定の詳細は build_example.rs および ffi_example.cpp を参照してください。

/*
// 1. C++関数のシグネチャを宣言
unsafe extern "C" {
    fn create_processor() -> *mut std::ffi::c_void;
    fn process_audio(ptr: *mut std::ffi::c_void, value: f32) -> f32;
    fn destroy_processor(ptr: *mut std::ffi::c_void);
}

// 2. ポインタを安全に扱うためのラッパ構造体
struct SafeProcessor {
    ptr: *mut std::ffi::c_void,
}

// スレッド間送信を許可 (自己責任で安全性を担保すること)
unsafe impl Send for SafeProcessor {}

// Dropトレイトを実装し、メモリリークを防ぐ
impl Drop for SafeProcessor {
    fn drop(&mut self) {
        unsafe {
            println!("Destroying C++ processor...");
            destroy_processor(self.ptr);
        }
    }
}
*/

fn main() {
    println!("Initializing C++ FFI Example...");
    println!("Please refer to build_example.rs and ffi_example.cpp for actual linking steps.");

    /*
        // 3. C++オブジェクトの生成
        let ptr = unsafe { create_processor() };
        if ptr.is_null() {
            eprintln!("Failed to create processor.");
            return;
        }

        // RAIIパターンでポインタを管理
        let processor = SafeProcessor { ptr };

        // 4. C++関数の呼び出し
        let input = 0.5;
        let output = unsafe { process_audio(processor.ptr, input) };

        println!("Processed value: {} -> {}", input, output);

        // processorがスコープを抜けると自動的にDropが呼ばれ、destroy_processorが実行される
    */
}
