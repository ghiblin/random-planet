Feature: Mesh construction and validation

  Scenario: Constructing a Mesh with all triangle indices in bounds succeeds
    Given a list of 3 vertices
    And a Triangle referencing indices 0, 1, 2
    When a Mesh is constructed from the vertices and the triangle
    Then the Mesh is constructed successfully
    And the Mesh's vertices match the given list
    And the Mesh's triangles match the given list

  Scenario: Constructing a Mesh with an out-of-bounds triangle index fails
    Given a list of 3 vertices
    And a Triangle referencing indices 0, 1, 3
    When a Mesh is constructed from the vertices and the triangle
    Then the construction fails with a vertex-index-out-of-bounds error

  Scenario: Constructing an empty Mesh succeeds
    Given an empty list of vertices
    And an empty list of triangles
    When a Mesh is constructed from the vertices and the triangles
    Then the Mesh is constructed successfully
    And the Mesh has zero vertices
    And the Mesh has zero triangles

  Scenario: Constructing a cube mesh with side 1.0 succeeds with the expected vertex and triangle counts
    Given a Mesh constructed by Mesh::cube with side 1.0
    Then the Mesh is constructed successfully
    And the Mesh has 8 vertices
    And the Mesh has 12 triangles

  Scenario: Every triangle in a cube mesh references three distinct vertex indices
    Given a Mesh constructed by Mesh::cube with side 1.0
    Then every triangle in the Mesh has three distinct vertex indices
    And every triangle index in the Mesh is less than 8

  Scenario: Constructing a cube mesh with side 2.0 doubles the distance from the origin to every vertex
    Given a Mesh constructed by Mesh::cube with side 1.0
    And a Mesh constructed by Mesh::cube with side 2.0
    Then every vertex of the side-2.0 Mesh is twice as far from the origin as the corresponding vertex of the side-1.0 Mesh

  Scenario: Constructing a cube mesh with a negative side fails
    When a Mesh is constructed by Mesh::cube with side -1.0
    Then the construction fails with a negative-cube-side error

  Scenario: Constructing a cube mesh with side 0.0 produces a degenerate mesh with all vertices at the origin
    Given a Mesh constructed by Mesh::cube with side 0.0
    Then the Mesh is constructed successfully
    And every vertex of the Mesh is at the origin
