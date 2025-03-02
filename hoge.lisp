(define edgelen 10)
(define negedgelen (- 0 edgelen))
(preview (to_mesh (linear_extrude 20 (turtle (p 0 0) (p edgelen 0) (p 0 negedgelen)))))
(preview (to_mesh (linear_extrude 20 (turtle (p 0 0) (p edgelen 0) (p 0 negedgelen)))))
