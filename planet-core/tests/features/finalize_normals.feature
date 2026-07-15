Feature: Computing Face and Vertex normals from final mesh geometry

  Scenario: Finalizing normals on a cube Mesh gives every face its flat normal and every vertex an area-weighted average
    Given a Mesh constructed by Mesh::cube with side 1.0
    When normals are finalized
    Then every face's normal has unit length
    And vertex 0's normal is approximately (-0.577, -0.577, -0.577)

  Scenario: A vertex shared by faces of unequal area weights its normal toward the larger face
    Given a Mesh where vertex 0 is shared by one large face facing (0.0, 0.0, 1.0) and one small face facing (1.0, 0.0, 0.0)
    When normals are finalized
    Then vertex 0's normal is approximately (0.01, 0.0, 1.0)

  Scenario: A vertex referenced only by degenerate faces never panics and falls back to a zero normal
    Given a Mesh with 3 vertices at the same position
    And a triangle index-triple (0, 1, 2)
    When normals are finalized
    Then no panic occurs
    And every vertex's normal is (0.0, 0.0, 0.0)
