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
              (call-with-exception-handler
                (lambda (e)
                  (display "Error in OSC hook for path: ")
                  (display path)
                  (display " - ")
                  (displayln e)
                  #f)
                (lambda ()
                  (callback path args))))
            *osc-hooks*))

;; プラグインスクリプトの評価はRust側のディレクトリ走査によって自動的に行われます。
