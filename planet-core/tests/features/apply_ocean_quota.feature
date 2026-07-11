Feature: Flattening a mesh's lowest-radius vertices to a shared sea level

  Scenario: Flattening raises every vertex below the computed sea level to a shared radius
    Given a Mesh with vertices at radii 0.9, 1.0, 1.1, 1.2
    When that mesh is flattened with an OceanQuota of 0.5
    Then the resulting Mesh has vertex radii 1.1, 1.1, 1.1, 1.2

  Scenario: Flattening with an OceanQuota of 0.0 leaves the mesh unchanged
    Given a Mesh with vertices at radii 0.9, 1.0, 1.1
    When that mesh is flattened with an OceanQuota of 0.0
    Then the resulting Mesh is identical to the original mesh

  Scenario: Flattening with an OceanQuota of 1.0 raises every vertex to the mesh's maximum radius
    Given a Mesh with vertices at radii 0.9, 1.0, 1.1
    When that mesh is flattened with an OceanQuota of 1.0
    Then the resulting Mesh has vertex radii 1.1, 1.1, 1.1

  Scenario: Flattening preserves vertex count and triangle topology
    Given an icosahedron mesh
    When the icosahedron mesh is flattened with an OceanQuota of 0.4
    Then the resulting Mesh has 12 vertices
    And the resulting Mesh has the same triangles as the icosahedron mesh

  Scenario: Flattening a mesh with all-equal radii is a no-op
    Given an icosahedron mesh
    When the icosahedron mesh is flattened with an OceanQuota of 0.4
    Then the resulting Mesh is identical to the icosahedron mesh

  Scenario: Flattening never panics when a vertex sits exactly at the origin
    Given a Mesh with a vertex exactly at the origin
    When that mesh is flattened with an OceanQuota of 0.9
    Then no panic occurs

  Scenario: Flattening an empty mesh is a no-op
    Given a Mesh with no vertices and no triangles
    When that mesh is flattened with an OceanQuota of 0.5
    Then the resulting Mesh is identical to the original mesh
