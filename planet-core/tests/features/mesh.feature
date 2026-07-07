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
