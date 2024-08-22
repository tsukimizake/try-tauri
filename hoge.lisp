(def (fix n) (if (zero? n) 1 (* n (fix (- n 1)))))
