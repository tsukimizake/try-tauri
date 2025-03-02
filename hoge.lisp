(define edgelen 10)
(define negedgelen (- 0 edgelen))

(-> (turtle (p 0 0) (p edgelen 0) (p 0 negedgelen))
    (linear_extrude 20)
    (to_mesh)
    (preview))

