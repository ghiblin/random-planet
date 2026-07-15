Feature: Shaping a mesh's elevation from a continuous noise field

  Scenario: Applying terrain noise never displaces a vertex beyond amplitude bounds
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then every vertex of the resulting Mesh has a radius less than or equal to 1.2
    And every vertex of the resulting Mesh has a radius greater than or equal to 0.8

  Scenario: Applying terrain noise is deterministic for a given seed
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise, producing the first Mesh
    And terrain noise is applied to the same icosahedron mesh with seed 7 and that TerrainNoise, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: Applying terrain noise with different seeds produces different vertex positions
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise, producing the first Mesh
    And terrain noise is applied to the same icosahedron mesh with seed 99 and that TerrainNoise, producing the second Mesh
    Then the first Mesh and the second Mesh are not identical

  Scenario: Applying terrain noise with seeds that agree on their low 32 bits produces identical terrain
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise, producing the first Mesh
    And terrain noise is applied to the same icosahedron mesh with seed 4294967303 and that TerrainNoise, producing the second Mesh
    Then the first Mesh and the second Mesh are identical

  Scenario: Applying terrain noise with zero amplitude leaves every vertex radius unchanged
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.0
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then every vertex of the resulting Mesh has a radius equal to the corresponding vertex's radius in the icosahedron mesh

  Scenario: Applying terrain noise with terrace levels set produces radii clustered at a bounded number of distinct values
    Given an icosahedron mesh subdivided 3 steps with SubdivisionMode::UniformRedSplit and seed 7
    And a TerrainNoise with amplitude 0.3 and 6 terrace levels
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then the resulting Mesh has at most 6 distinct vertex radii, within floating-point tolerance

  Scenario: Applying terrain noise never panics when a vertex sits exactly at the origin
    Given a Mesh with a vertex exactly at the origin
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then no panic occurs

  Scenario: Applying terrain noise to an empty mesh is a no-op
    Given a Mesh with no vertices and no triangles
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then the resulting Mesh is identical to the original mesh

  Scenario: Applying terrain noise preserves vertex count and face topology
    Given an icosahedron mesh
    And a TerrainNoise with amplitude 0.2
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then the resulting Mesh has 12 vertices
    And the resulting Mesh has the same faces as the icosahedron mesh

  Scenario: Terrain noise with zero amplitude produces a geodesic sphere with no degenerate sliver triangles
    Given an icosahedron mesh subdivided 8 steps with SubdivisionMode::UniformRedSplit and seed 7
    And a TerrainNoise with amplitude 0.0
    When terrain noise is applied to that mesh with seed 7 and that TerrainNoise
    Then every face in the resulting Mesh has all 3 angles between 8 and 155 degrees
