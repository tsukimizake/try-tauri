(define edgelen 50)
(define negedgelen (- 0 edgelen))
(define width 10)
(define negwidth (- 0 width))


(define hori_footprint 
  (-> (turtle (p 0 0) (p edgelen 0) (p 0 width) (p negedgelen 0))
      (linear_extrude 20)
      ))

(define vert_footprint
  (-> 
    (rotate hori_footprint (p (/ edgelen 2) (/ width 2)) 'z 90)
    (translate 0 0 20)))


(-> 
  vert_footprint
  (or hori_footprint)
  (preview))
