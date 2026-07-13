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
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7, the default ElevationNoiseRange, and the default NormalNoiseRange
    Then the resulting Mesh has 80 triangles

  Scenario: Subdividing the icosahedron mesh by 2 steps using SubdivisionMode::RadialRandomSplit grows the triangle count geometrically
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7, the default ElevationNoiseRange, and the default NormalNoiseRange
    Then the resulting Mesh has 320 triangles

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RadialRandomSplit does not duplicate vertices at shared edges
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7, the default ElevationNoiseRange, and the default NormalNoiseRange
    Then the resulting Mesh has 42 vertices

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RadialRandomSplit never creates cracks between adjacent triangles
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7, the default ElevationNoiseRange, and the default NormalNoiseRange
    Then no two vertices in the resulting Mesh have the same position

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RadialRandomSplit keeps every vertex radius within the configured bound
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, and the default NormalNoiseRange
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.27
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.05

  Scenario: Subdividing with 0 steps using SubdivisionMode::RadialRandomSplit leaves the mesh unchanged
    Given an icosahedron mesh
    When the mesh is subdivided with 0 steps using SubdivisionMode::RadialRandomSplit with seed 7, the default ElevationNoiseRange, and the default NormalNoiseRange
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: SubdivisionMode::RadialRandomSplit never displaces the mesh's original vertices
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, and the default NormalNoiseRange
    Then the first 12 vertices of the resulting Mesh have the same positions as the icosahedron mesh's vertices

  Scenario: SubdivisionMode::RadialRandomSplit is deterministic for a given seed
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7, the default ElevationNoiseRange, and the default NormalNoiseRange, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::RadialRandomSplit with seed 7, the default ElevationNoiseRange, and the default NormalNoiseRange, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: SubdivisionMode::RadialRandomSplit with different seeds produces different vertex positions
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, and the default NormalNoiseRange, producing the first Mesh
    And the same icosahedron mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 99, an ElevationNoiseRange of low -0.1 and high 0.1, and the default NormalNoiseRange, producing the second Mesh
    Then the first Mesh and the second Mesh are not identical

  Scenario: SubdivisionMode::RadialRandomSplit with a zero-width ElevationNoiseRange at zero and a zero-width NormalNoiseRange at zero behaves like SubdivisionMode::UniformRedSplit
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, and a NormalNoiseRange of low 0.0 and high 0.0, producing the first Mesh
    And the same icosahedron mesh is subdivided with 1 step using SubdivisionMode::UniformRedSplit, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: SubdivisionMode::RadialRandomSplit never panics when an edge's midpoint is exactly the origin
    Given a Mesh with an edge whose midpoint is the origin
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7, the default ElevationNoiseRange, and the default NormalNoiseRange
    Then no panic occurs

  Scenario: SubdivisionMode::RadialRandomSplit keeps a new vertex exactly coplanar when NormalNoiseRange is zero-width at zero
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, and a NormalNoiseRange of low 0.0 and high 0.0
    Then the new vertex on edge 1-2 is coplanar with vertices 1, 2, and the origin

  Scenario: SubdivisionMode::RadialRandomSplit moves a new vertex off the shared plane when NormalNoiseRange is non-zero
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RadialRandomSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, and a NormalNoiseRange of low 0.05 and high 0.05
    Then the new vertex on edge 1-2 is not coplanar with vertices 1, 2, and the origin

  Scenario: Subdividing the icosahedron mesh by 1 step using SubdivisionMode::RedGreenSplit with a min-edge-length below the icosahedron's own edge length quadruples the triangle count
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, the default NormalNoiseRange, a MinEdgeLength of 0.5, and a SplitPointVariance of 0.0
    Then the resulting Mesh has 80 triangles

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RedGreenSplit does not duplicate vertices at shared edges
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, the default NormalNoiseRange, a MinEdgeLength of 0.5, and a SplitPointVariance of 0.0
    Then the resulting Mesh has 42 vertices

  Scenario: Subdividing the icosahedron mesh with SubdivisionMode::RedGreenSplit never creates cracks between red and green triangles
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a NormalNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 2.1, and a SplitPointVariance of 0.0
    Then no two vertices in the resulting Mesh have the same position

  Scenario: SubdivisionMode::RedGreenSplit keeps every vertex radius within the configured bound
    Given an icosahedron mesh
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, the default NormalNoiseRange, a MinEdgeLength of 0.5, and a SplitPointVariance of 0.0
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.16
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.05

  Scenario: SubdivisionMode::RedGreenSplit's vertex radius bound does not grow with additional subdivision rounds
    Given an icosahedron mesh
    When the mesh is subdivided with 8 steps using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, the default NormalNoiseRange, a MinEdgeLength of 0.05, and a SplitPointVariance of 0.0
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.46
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.05

  Scenario: All 3 edges above the threshold produce a red split with 4 recursable children
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a NormalNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 1.5, and a SplitPointVariance of 0.0
    Then the resulting Mesh has 4 triangles
    And the resulting Mesh has 6 vertices

  Scenario: Exactly 2 edges above the threshold produce a green split with 3 non-recursable children fanned through their two midpoints
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a NormalNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 2.1, and a SplitPointVariance of 0.0
    Then the resulting Mesh has 3 triangles
    And the resulting Mesh has 5 vertices

  Scenario: Exactly 1 edge above the threshold produces a green split with 2 non-recursable children
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a NormalNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 2.5, and a SplitPointVariance of 0.0
    Then the resulting Mesh has 2 triangles
    And the resulting Mesh has 4 vertices

  Scenario: No edge above the threshold produces an unchanged leaf triangle
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a NormalNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 3.5, and a SplitPointVariance of 0.0
    Then the resulting Mesh is identical to the source Mesh

  Scenario: Subdivision naturally stops growing once every edge in the mesh is below the threshold, even if more steps are requested
    Given an icosahedron mesh
    When the mesh is subdivided with 3 steps using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, a NormalNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 0.35, and a SplitPointVariance of 0.0, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, a NormalNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 0.35, and a SplitPointVariance of 0.0, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: SubdivisionMode::RedGreenSplit with a MinEdgeLength of 0.0 and a SplitPointVariance of 0.0 behaves like SubdivisionMode::UniformRedSplit
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a NormalNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 0.0, and a SplitPointVariance of 0.0, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::UniformRedSplit, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: SubdivisionMode::RedGreenSplit is deterministic for a given seed
    Given an icosahedron mesh
    When the mesh is subdivided with 2 steps using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, the default NormalNoiseRange, a MinEdgeLength of 0.35, and a SplitPointVariance of 0.1, producing the first Mesh
    And the same icosahedron mesh is subdivided with 2 steps using SubdivisionMode::RedGreenSplit with seed 7, the default ElevationNoiseRange, the default NormalNoiseRange, a MinEdgeLength of 0.35, and a SplitPointVariance of 0.1, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: A non-zero SplitPointVariance moves the split point off the exact midpoint
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low 0.0 and high 0.0, a NormalNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 1.5, and a SplitPointVariance of 0.3
    Then no vertex in the resulting Mesh sits at the exact midpoint of edge 0-1

  Scenario: SubdivisionMode::RedGreenSplit keeps a new vertex exactly coplanar when NormalNoiseRange is zero-width at zero
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, a NormalNoiseRange of low 0.0 and high 0.0, a MinEdgeLength of 2.5, and a SplitPointVariance of 0.0
    Then the new vertex on edge 1-2 is coplanar with vertices 1, 2, and the origin

  Scenario: SubdivisionMode::RedGreenSplit moves a new vertex off the shared plane when NormalNoiseRange is non-zero
    Given a Mesh with 3 vertices at the corners of an arbitrary triangle
    And a Triangle referencing indices 0, 1, 2
    When the mesh is subdivided with 1 step using SubdivisionMode::RedGreenSplit with seed 7, an ElevationNoiseRange of low -0.1 and high 0.1, a NormalNoiseRange of low 0.05 and high 0.05, a MinEdgeLength of 2.5, and a SplitPointVariance of 0.0
    Then the new vertex on edge 1-2 is not coplanar with vertices 1, 2, and the origin
