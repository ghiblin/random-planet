Feature: Base icosahedron construction

  Scenario: Constructing the icosahedron produces the expected vertex and triangle counts
    Given an icosahedron mesh
    Then the Mesh is constructed successfully
    And the Mesh has 12 vertices
    And the Mesh has 20 triangles

  Scenario: Every vertex of the icosahedron mesh lies on the unit sphere
    Given an icosahedron mesh
    Then every vertex of the Mesh has a radius of 1.0

  Scenario: Every triangle in the icosahedron mesh references three distinct vertex indices
    Given an icosahedron mesh
    Then every triangle in the Mesh has three distinct vertex indices
    And every triangle index in the Mesh is less than 12

  Scenario: Every triangle in the icosahedron mesh is wound outward
    Given an icosahedron mesh
    Then every triangle's face normal points away from the origin
