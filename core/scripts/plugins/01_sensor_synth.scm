(define (sensor-synth:map address args)
  (if (equal? address "/sensor/val")
      (if (not (null? args))
          (let ((val (string->number (car args))))
            (if (number? val)
                (sound:set-param "frequency" (* val 1000.0)))))))

(register-osc-hook! sensor-synth:map)
