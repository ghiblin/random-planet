Feature: Scrambling a mesh's existing vertices along all three axes

  Scenario: Scrambling the icosahedron mesh's vertices changes the resulting Mesh
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1
    Then the resulting Mesh is not identical to the icosahedron mesh

  Scenario: Scrambling preserves vertex count and face topology
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1
    Then the resulting Mesh has 12 vertices
    And the resulting Mesh has the same faces as the icosahedron mesh

  Scenario: Scrambling with a zero-width VertexScrambleRange at zero leaves the mesh unchanged
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low 0.0 and high 0.0
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: Scrambling is deterministic for a given seed
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1, producing the first Mesh
    And the same icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: Scrambling with different seeds produces different vertex positions
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1, producing the first Mesh
    And the same icosahedron mesh is scrambled with seed 99 and a VertexScrambleRange of low -0.1 and high 0.1, producing the second Mesh
    Then the first Mesh and the second Mesh are not identical

  Scenario: Scrambling never pushes a vertex below the minimum vertex radius
    Given a Mesh with a vertex at position 10.0, 10.0, 10.0
    When that mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.999 and high -0.999
    Then every vertex of the resulting Mesh has a radius greater than or equal to 0.05

  Scenario: Scrambling never panics when a vertex sits exactly at the origin
    Given a Mesh with a vertex exactly at the origin
    When that mesh is scrambled with seed 7 and a VertexScrambleRange of low 0.0 and high 0.0
    Then no panic occurs

  Scenario: Scrambling moves a vertex off a coordinate plane it started on
    Given an icosahedron mesh
    When the icosahedron mesh is scrambled with seed 7 and a VertexScrambleRange of low 0.02 and high 0.02
    Then no vertex of the resulting Mesh has a coordinate equal to 0.0

  Scenario: Scrambling an arbitrary mesh proves it is not icosahedron-specific
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a triangle index-triple (0, 1, 2)
    When that mesh is scrambled with seed 7 and a VertexScrambleRange of low -0.1 and high 0.1
    Then the resulting Mesh has 3 vertices
