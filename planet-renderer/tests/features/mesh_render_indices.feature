Feature: Converting a Mesh into render indices

  Scenario: Converting a cube Mesh into render indices produces sequential indices
    Given a Mesh constructed by Mesh::cube with side 1.0
    When the mesh is converted into render indices
    Then the render index list is 0 through 35 in order

  Scenario: Converting an empty Mesh into render indices produces an empty list
    Given an empty Mesh with no vertices and no triangles
    When the mesh is converted into render indices
    Then the render index list is empty

  Scenario: Converting a Mesh with enough triangles to exceed u16's range produces correct indices
    Given a Mesh with 30000 triangles
    When the mesh is converted into render indices
    Then the render index list has 90000 indices
    And the last render index is 89999
