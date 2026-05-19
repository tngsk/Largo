(define (sensor-synth:map address args)
  (if (equal? address "/sensor/val")
      (let ((val (string->number (car args))))
        (sound:set-param "freq" (* val 1000.0)))))

(register-osc-hook! sensor-synth:map)
