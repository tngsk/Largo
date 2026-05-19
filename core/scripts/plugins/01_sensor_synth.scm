;; =================================================================
;; 拡張プラグイン: センサーマッピング & フィルター変調
;; =================================================================

;; プレフィックスを伴う関数定義
(define (plugin:map-range x in-min in-max out-min out-max)
  (+ out-min (* (/ (- x in-min) (- in-max in-min)) (- out-max out-min))))

;; 自律レジストリ登録（メインルーティングを汚染しない）
(register-osc-hook!
 (lambda (path args)
   (cond
     ;; センサーAのOSCアドレスをインターセプト
     ((string=? path "/sensor/distance")
      (let* ((raw-val (if (string? (car args)) (string->number (car args)) (car args))) ; 0.0 から 1.0 の想定
             (target-freq (if raw-val (plugin:map-range raw-val 0.0 1.0 200.0 5000.0) 200.0)))
        ;; Rust経由でRNBOの"cutoff"パラメータを操作
        (sound:set-param "cutoff" target-freq)))

     ;; 別のセンサーアドレスへのフック追加も、この条件節を増やすか別ファイルに分けるだけ
     ((string=? path "/sensor/intensity")
      (let ((gain (if (string? (car args)) (string->number (car args)) (car args))))
        (if gain
            (sound:set-param "gain" gain)
            #f)))

     (else #f))))
