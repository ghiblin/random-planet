Feature: Base icosahedron construction

  Scenario: Constructing the icosahedron produces the expected vertex and face counts
    Given an icosahedron mesh
    Then the Mesh is constructed successfully
    And the Mesh has 12 vertices
    And the Mesh has 20 faces

  Scenario: Every vertex of the icosahedron mesh lies on the unit sphere
    Given an icosahedron mesh
    Then every vertex of the Mesh has a radius of 1.0

  Scenario: Every face in the icosahedron mesh references three distinct vertex indices
    Given an icosahedron mesh
    Then every face in the Mesh has three distinct vertex indices
    And every face's vertex index in the Mesh is less than 12

  Scenario: Every face in the icosahedron mesh is wound outward
    Given an icosahedron mesh
    Then every face's normal points away from the origin
