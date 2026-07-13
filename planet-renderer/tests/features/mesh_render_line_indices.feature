Feature: Converting a Mesh into wireframe line-list render indices

  Scenario: Converting a cube Mesh into line indices produces edge pairs per triangle
    Given a Mesh constructed by Mesh::cube with side 1.0
    When the mesh is converted into wireframe line indices
    Then the wireframe line index list has 72 indices
    And the wireframe line indices for the first triangle are 0, 1, 1, 2, 2, 0

  Scenario: Converting an empty Mesh into wireframe line indices produces an empty list
    Given an empty Mesh with no vertices and no triangles
    When the mesh is converted into wireframe line indices
    Then the wireframe line index list is empty

  Scenario: Converting a Mesh with enough triangles to exceed u16's range produces correct line indices
    Given a Mesh with 30000 triangles
    When the mesh is converted into wireframe line indices
    Then the wireframe line index list has 180000 indices
    And the last wireframe line index is 89997
