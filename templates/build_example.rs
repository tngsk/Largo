// build_example.rs
// C++コードをコンパイルしてリンクするためのbuild.rsのテンプレート
// 実際のプロジェクトではこのファイルをビルドルートに置き、ccクレートをbuild-dependenciesに追加します。

fn main() {
    // 依存するファイルが変更されたら再ビルドするようcargoに指示
    println!("cargo:rerun-if-changed=ffi_example.cpp");

    // ccクレートを使ってC++コードをコンパイル
    /*
    cc::Build::new()
        .cpp(true)                 // C++コンパイラを使用
        .std("c++11")              // C++11標準を使用
        .file("ffi_example.cpp")   // コンパイル対象ファイル
        .compile("ffi_example");   // 出力ライブラリ名
    */
    // ※ このテンプレートではコメントアウトしていますが、
    // 実際に使用する場合はCargo.tomlに `cc = "1.0"` を追加し、コメントを外してください。
}
