Feature: Recursive subdivision via the SubdivisionMode facade

  Scenario: Subdividing the icosahedron mesh by 1 step using SubdivisionMode::UniformRedSplit quadruples the triangle count
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit
    Then the resulting Mesh has 80 triangles

  Scenario: Subdividing the icosahedron mesh by 2 steps grows the triangle count geometrically
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit
    Then the resulting Mesh has 320 triangles

  Scenario: Subdividing the icosahedron mesh by 1 step does not duplicate vertices at shared edges
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit
    Then the resulting Mesh has 42 vertices

  Scenario: Subdividing the icosahedron mesh never creates cracks between adjacent triangles
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit
    Then no two vertices in the resulting Mesh have the same position

  Scenario: Subdividing the icosahedron mesh never pushes vertices beyond the base radius
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.0

  Scenario: A new vertex sits at the exact arithmetic mean of its edge's endpoints
    Given an icosahedron mesh
    And the two vertices of the first triangle's first edge in the icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit
    Then a vertex exists in the resulting Mesh at the exact midpoint of the two given vertices

  Scenario: SubdivisionMode::UniformRedSplit subdivides an arbitrary single-triangle mesh, proving subdivide is not icosahedron-specific
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit
    Then the resulting Mesh has 4 triangles
    And the resulting Mesh has 6 vertices

  Scenario: Subdividing with 0 steps leaves the mesh unchanged
    Given an icosahedron mesh
    When the mesh is subdivided with 0 steps using SubdivisionMode::UniformRedSplit
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: Omitting steps and mode falls back to the default of 3 steps using the default SubdivisionMode
    Given an icosahedron mesh
    When the mesh is subdivided with default SubdivisionArgs
    Then the resulting Mesh has 1280 triangles

  Scenario: The update callback is invoked once per completed round with that round's mesh
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit and a recording update callback
    Then the update callback was invoked 2 times
    And the update callback's 1st invocation received a Mesh with 80 triangles
    And the update callback's 2nd invocation received a Mesh with 320 triangles

  Scenario: Subdividing with 0 steps never invokes the update callback
    Given an icosahedron mesh
    When the mesh is subdivided with 0 steps using SubdivisionMode::UniformRedSplit and a recording update callback
    Then the update callback was invoked 0 times

  Scenario: Subdividing the icosahedron mesh by 1 step using SubdivisionMode::RadialRandomSplit quadruples the triangle count
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange
    Then the resulting Mesh has 80 triangles

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RadialRandomSplit does not duplicate vertices at shared edges
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange
    Then the resulting Mesh has 42 vertices

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RadialRandomSplit never creates cracks between adjacent triangles
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange
    Then no two vertices in the resulting Mesh have the same position

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RadialRandomSplit keeps every vertex radius within the configured bound
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7 and an ElevationNoiseRange of low -0.1 and high 0.1
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.2
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.05

  Scenario: Subdividing with 0 steps using SubdivisionMode::RadialRandomSplit leaves the mesh unchanged
    Given an icosahedron mesh
    When the mesh is subdivided with 0 steps using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: SubdivisionMode::RadialRandomSplit never displaces the mesh's original vertices
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and an ElevationNoiseRange of low -0.1 and high 0.1
    Then the first 12 vertices of the resulting Mesh have the same positions as the icosahedron mesh's vertices

  Scenario: SubdivisionMode::RadialRandomSplit is deterministic for a given seed
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7 and the default ElevationNoiseRange, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: SubdivisionMode::RadialRandomSplit with different seeds produces different vertex positions
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and an ElevationNoiseRange of low -0.1 and high 0.1, producing the first Mesh
    And the same icosahedron mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 99 and an ElevationNoiseRange of low -0.1 and high 0.1, producing the second Mesh
    Then the first Mesh and the second Mesh are not identical

  Scenario: SubdivisionMode::RadialRandomSplit with a zero-width ElevationNoiseRange at zero behaves like SubdivisionMode::UniformRedSplit
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7 and an ElevationNoiseRange of low 0.0 and high 0.0, producing the first Mesh
    And the same icosahedron mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit, producing the second Mesh
    Then the first Mesh and the second Mesh are identical
