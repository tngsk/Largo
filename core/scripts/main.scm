;; =================================================================
;; 不変層: メイン・イベント・ディスパッチャ
;; =================================================================

(define *osc-hooks* '())

;; 各プラグインが初期化時に呼び出すレジストリAPI
(define (register-osc-hook! callback)
  (set! *osc-hooks* (cons callback *osc-hooks*)))

;; Rust側からメッセージ到達時に一律で叩かれる配信エントリポイント
(define (on-osc-event path args)
  (for-each (lambda (callback)
              (callback path args))
            *osc-hooks*))

;; プラグインスクリプトをここから評価（実際はディレクトリ走査推奨）
(load "scripts/plugins/01_sensor_synth.scm")
(load "scripts/plugins/02_tutorial_demo.scm")
