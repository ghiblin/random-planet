Feature: Converting a Mesh into flat-shaded render vertices

  Scenario: Converting a cube Mesh into render vertices produces one vertex per triangle corner
    Given a Mesh constructed by Mesh::cube with side 1.0
    When the mesh is converted into render vertices
    Then the render vertex list has 36 vertices
    And every triangle's three render vertices share an identical normal
    And every render vertex normal has unit length

  Scenario: Converting the cube mesh's +X face triangles produces an outward-facing normal
    Given a Mesh constructed by Mesh::cube with side 1.0
    When the mesh is converted into render vertices
    Then the +X face triangles have the normal (1.0, 0.0, 0.0)

  Scenario: Converting a Mesh with a degenerate triangle never panics and yields a zero normal
    Given a Mesh with 3 vertices at the same position
    And a Triangle referencing indices 0, 1, 2
    When the mesh is converted into render vertices
    Then no panic occurs
    And every render vertex normal is (0.0, 0.0, 0.0)

  Scenario: Converting an empty Mesh into render vertices produces an empty list
    Given an empty Mesh with no vertices and no triangles
    When the mesh is converted into render vertices
    Then the render vertex list is empty
