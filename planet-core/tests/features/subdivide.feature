Feature: Recursive subdivision via a pluggable SubdivisionStrategy

  Scenario: Subdividing the icosahedron mesh once with the uniform red-split strategy quadruples the triangle count
    Given an icosahedron mesh
    When the mesh is subdivided to depth 1 using the uniform red-split strategy
    Then the resulting Mesh has 80 triangles

  Scenario: Subdividing the icosahedron mesh to depth 2 grows the triangle count geometrically
    Given an icosahedron mesh
    When the mesh is subdivided to depth 2 using the uniform red-split strategy
    Then the resulting Mesh has 320 triangles

  Scenario: Subdividing the icosahedron mesh once does not duplicate vertices at shared edges
    Given an icosahedron mesh
    When the mesh is subdivided to depth 1 using the uniform red-split strategy
    Then the resulting Mesh has 42 vertices

  Scenario: Subdividing the icosahedron mesh never creates cracks between adjacent triangles
    Given an icosahedron mesh
    When the mesh is subdivided to depth 2 using the uniform red-split strategy
    Then no two vertices in the resulting Mesh have the same position

  Scenario: Subdividing the icosahedron mesh never pushes vertices beyond the base radius
    Given an icosahedron mesh
    When the mesh is subdivided to depth 2 using the uniform red-split strategy
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.0

  Scenario: A new vertex sits at the exact arithmetic mean of its edge's endpoints
    Given an icosahedron mesh
    And the two vertices of the first triangle's first edge in the icosahedron mesh
    When the mesh is subdivided to depth 1 using the uniform red-split strategy
    Then a vertex exists in the resulting Mesh at the exact midpoint of the two given vertices

  Scenario: Subdividing to depth 0 leaves the mesh unchanged regardless of strategy
    Given an icosahedron mesh
    When the mesh is subdivided to depth 0 using the uniform red-split strategy
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: The uniform red-split strategy subdivides an arbitrary single-triangle mesh, proving subdivide is not icosahedron-specific
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided to depth 1 using the uniform red-split strategy
    Then the resulting Mesh has 4 triangles
    And the resulting Mesh has 6 vertices
