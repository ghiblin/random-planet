Feature: Recursive subdivision via the SubdivisionMode facade

  Scenario: Subdividing the icosahedron mesh by 1 step using SubdivisionMode::UniformRedSplit quadruples the face count
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit with seed 7
    Then the resulting Mesh has 80 faces

  Scenario: Subdividing the icosahedron mesh by 2 steps grows the face count geometrically
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7
    Then the resulting Mesh has 320 faces

  Scenario: Subdividing the icosahedron mesh by 1 step does not duplicate vertices at shared edges
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit with seed 7
    Then the resulting Mesh has 42 vertices

  Scenario: Subdividing the icosahedron mesh never creates cracks between adjacent triangles
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7
    Then no two vertices in the resulting Mesh have the same position

  Scenario: Subdividing the icosahedron mesh never pushes vertices beyond the base radius
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.0

  Scenario: Subdividing the icosahedron mesh never pushes a vertex below a safe minimum radius
    Given an icosahedron mesh
    When the mesh is subdivided with 6 steps using SubdivisionMode::UniformRedSplit with seed 7
    Then every vertex of the resulting Mesh has a radius greater than or equal to 0.7

  Scenario: A new vertex is displaced from its edge's exact midpoint, bounded by the edge's length
    Given an icosahedron mesh
    And the two vertices of the first face's first edge in the icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit with seed 7
    Then a vertex exists in the resulting Mesh within 0.06 times the edge's length of the exact midpoint of the two given vertices
    And no vertex in the resulting Mesh sits at the exact midpoint of the two given vertices

  Scenario: Subdividing the icosahedron mesh is deterministic for a given seed
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: Subdividing the icosahedron mesh with different seeds produces different vertex positions
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 99, producing the second Mesh
    Then the first Mesh and the second Mesh are not identical

  Scenario: SubdivisionMode::UniformRedSplit subdivides an arbitrary single-triangle mesh, proving subdivide is not icosahedron-specific
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a triangle index-triple (0, 1, 2)
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit with seed 7
    Then the resulting Mesh has 4 faces
    And the resulting Mesh has 6 vertices

  Scenario: Subdividing with 0 steps leaves the mesh unchanged
    Given an icosahedron mesh
    When the mesh is subdivided with 0 steps using SubdivisionMode::UniformRedSplit with seed 7
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: Omitting steps and mode falls back to the default of 3 steps using the default SubdivisionMode
    Given an icosahedron mesh
    When the mesh is subdivided with default SubdivisionArgs
    Then the resulting Mesh has 1280 faces

  Scenario: The update callback is invoked once per completed round with that round's mesh
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit with seed 7 and a recording update callback
    Then the update callback was invoked 2 times
    And the update callback's 1st invocation received a Mesh with 80 faces
    And the update callback's 2nd invocation received a Mesh with 320 faces

  Scenario: Subdividing with 0 steps never invokes the update callback
    Given an icosahedron mesh
    When the mesh is subdivided with 0 steps using SubdivisionMode::UniformRedSplit with seed 7 and a recording update callback
    Then the update callback was invoked 0 times
