;; plugin_template.scm
;; 教育/カスタマイズ用テンプレート: Lispプラグインスクリプト

;; このファイルは動的に読み込まれるSchemeスクリプトのテンプレートです。
;; 名前空間の衝突を防ぐため、関数名には「<プラグイン名>:<関数名>」という規則を用います。
;; 例: (define (my-plugin:process ...))

;; 1. 他のプラグインに依存する場合の宣言 (トポロジカルソート用)
;; @require: base_plugin.scm

;; 2. メインの処理関数の定義
;; address: 受信したOSCアドレスの文字列 (例: "/my/val")
;; args: 受信した引数のリスト (例: ("0.5"))
(define (my-plugin:handle-osc address args)
  (if (equal? address "/custom/trigger")
      ;; 引数が存在するか明示的にチェック
      (if (not (null? args))
          (let ((val (string->number (car args))))
            ;; 型変換が成功したかチェック
            (if (number? val)
                (begin
                  ;; C++ (RNBO) 側で定義されたパラメータ名を指定してノンブロッキング送信
                  (sound:set-param "frequency" (* val 500.0))
                  (sound:set-param "amplitude" 0.8))
                (display "Warning: val is not a number\n"))))))

;; 3. OSCフックへの登録
;; main.scm のルーターがこの関数を呼び出せるように登録します
(register-osc-hook! my-plugin:handle-osc)
