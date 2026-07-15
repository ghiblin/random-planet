Feature: Mesh construction and validation

  Scenario: Constructing a Mesh with all triangle indices in bounds succeeds
    Given a list of 3 positions
    And a triangle index-triple (0, 1, 2)
    When a Mesh is constructed from the positions and the triangle index-triples
    Then the Mesh is constructed successfully
    And the Mesh's vertex positions match the given list
    And the Mesh has 1 face
    And that face has order 3
    And the Mesh has 3 edges
    And each of the 3 vertices has exactly 1 edge in its edges list

  Scenario: Constructing a Mesh with an out-of-bounds triangle index fails
    Given a list of 3 positions
    And a triangle index-triple (0, 1, 3)
    When a Mesh is constructed from the positions and the triangle index-triples
    Then the construction fails with a vertex-index-out-of-bounds error

  Scenario: Constructing an empty Mesh succeeds
    Given an empty list of positions
    And an empty list of triangle index-triples
    When a Mesh is constructed from the positions and the triangle index-triples
    Then the Mesh is constructed successfully
    And the Mesh has zero vertices
    And the Mesh has zero faces

  Scenario: Constructing a cube mesh with side 1.0 succeeds with the expected vertex and face counts
    Given a Mesh constructed by Mesh::cube with side 1.0
    Then the Mesh is constructed successfully
    And the Mesh has 8 vertices
    And the Mesh has 12 faces

  Scenario: Every face in a cube mesh references three distinct vertex indices
    Given a Mesh constructed by Mesh::cube with side 1.0
    Then every face in the Mesh has three distinct vertex indices
    And every face's vertex index in the Mesh is less than 8

  Scenario: Two faces sharing an edge each get their own Edge object, and the shared vertices see both incident faces
    Given a Mesh constructed by Mesh::cube with side 1.0
    Then vertex 0 has exactly 6 edges, one per incident face

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
