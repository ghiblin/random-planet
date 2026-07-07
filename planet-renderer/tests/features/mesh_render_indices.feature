Feature: Converting a Mesh into render indices

  Scenario: Converting a cube Mesh into render indices produces sequential indices
    Given a Mesh constructed by Mesh::cube with side 1.0
    When the mesh is converted into render indices
    Then the render index list is 0 through 35 in order

  Scenario: Converting an empty Mesh into render indices produces an empty list
    Given an empty Mesh with no vertices and no triangles
    When the mesh is converted into render indices
    Then the render index list is empty
