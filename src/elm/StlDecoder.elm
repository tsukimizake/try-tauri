module StlDecoder exposing (Stl, Vec)

--
-- STL FILE FORMAT
-- UINT8[80]    – Header                 -     80 bytes
-- UINT32       – Number of triangles    -      4 bytes
-- foreach triangle                      - 50 bytes:
--     REAL32[3] – Normal vector             - 12 bytes
--     REAL32[3] – Vertex 1                  - 12 bytes
--     REAL32[3] – Vertex 2                  - 12 bytes
--     REAL32[3] – Vertex 3                  - 12 bytes
--     UINT16    – Attribute byte count      -  2 bytes
-- end
--


type alias Vec =
    ( Float, Float, Float )


type alias Stl =
    { header : String
    , numTriangles : Int
    , triangles : List ( Vec, Vec, Vec )
    }
