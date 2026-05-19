;; =================================================================
;; 拡張プラグイン: チュートリアルデモ
;; =================================================================

;; チュートリアル用の簡易マップ関数
(define (tutorial:map-param x)
  (let ((val (if (string? x) (string->number x) x)))
    (if val
        (+ 100 (* val 100))
        100)))

;; チュートリアル用のOSCルーティング登録
(register-osc-hook!
 (lambda (path args)
   (cond
     ;; "/demo/freq" へのOSCメッセージを受け取る
     ((string=? path "/demo/freq")
      (let* ((raw-val (car args))
             (freq (tutorial:map-param raw-val)))
        ;; 周波数パラメータをRNBOに送る
        (sound:set-param "frequency" freq)))

     ;; "/demo/amp" へのOSCメッセージを受け取る
     ((string=? path "/demo/amp")
      (let* ((raw-val (car args))
             (amp (if (string? raw-val) (string->number raw-val) raw-val)))
        (if amp
            (sound:set-param "amplitude" amp)
            #f)))

     (else #f))))
