(define edgelen 10)
(define negedgelen (- 0 edgelen))
(define width 5)
(define negwidth (- 0 width))

(-> (turtle (p 0 0) (p edgelen 0) (p 0 width) (p negedgelen 0))
    (linear_extrude 20)
    (preview))

