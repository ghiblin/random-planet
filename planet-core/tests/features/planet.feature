Feature: Planet aggregate generation

  Scenario: Generating a Planet is deterministic for identical inputs
    Given a Planet generated with seed 42 and the Earthy preset at max depth 3
    When another Planet is generated with seed 42 and the Earthy preset at max depth 3
    Then the two Planets have identical meshes
    And the two Planets have identical colors

  Scenario: A different seed produces a different Planet
    Given a Planet generated with seed 42 and the Earthy preset at max depth 3
    When another Planet is generated with seed 43 and the Earthy preset at max depth 3
    Then the two Planets do not have identical meshes

  Scenario: Every vertex's color matches the preset's color gradient sampled at its radius
    Given a Planet generated with seed 7 and the Volcano preset at max depth 2
    Then every vertex's color in the resulting Planet equals the Volcano preset's color gradient sampled at that vertex's radius

  Scenario: Generating a Planet keeps every vertex radius within the preset's configured bound
    Given a Planet generated with seed 3 and the Rocky preset at max depth 2
    Then every vertex of the resulting Planet's mesh has a radius less than or equal to 1.22
    And every vertex of the resulting Planet's mesh has a radius greater than or equal to 0.78

  Scenario: A Planet generated at zero max depth keeps the icosahedron's topology, shaped by terrain noise
    Given a Planet generated with seed 1 and the Earthy preset at max depth 0
    Then the resulting Planet's mesh has 12 vertices
    And the resulting Planet's mesh has the same faces as the icosahedron mesh
    And the resulting Planet has exactly 12 colors

  Scenario: Subdivision depth deterministically produces the full geodesic face count for every preset
    Given a Planet generated with seed 5 and the Volcano preset at max depth 8
    Then the resulting Planet's mesh has exactly 1310720 faces

  Scenario: Increasing subdivision depth beyond 3 keeps growing an Earthy planet's mesh
    Given a Planet generated with seed 42 and the Earthy preset at max depth 3
    When another Planet is generated with seed 42 and the Earthy preset at max depth 5
    Then the second Planet's mesh has more vertices than the first Planet's mesh

  Scenario: The optional progress callback reports the base mesh and every subdivision round
    Given a recording progress callback
    When a Planet is generated with seed 9 and the Volcano preset at max depth 2 using that callback
    Then the progress callback was invoked 3 times
    And the progress callback's 1st invocation received round 0 with the base icosahedron mesh
    And the progress callback's 3rd invocation received a Mesh with 320 faces

  Scenario: The optional progress callback still reports the base mesh at zero max depth
    Given a recording progress callback
    When a Planet is generated with seed 9 and the Earthy preset at max depth 0 using that callback
    Then the progress callback was invoked 1 time
    And the progress callback's 1st invocation received round 0 with the base icosahedron mesh

  Scenario: Building a Planet with no fields set falls back to each field's default
    Given a Planet built with no fields set
    Then the resulting Planet's preset is Earthy
    And the resulting Planet's seed is 0
    And the resulting Planet's mesh has 12 vertices
    And the resulting Planet's mesh has the same faces as the icosahedron mesh
    And the resulting Planet's mesh is not identical to the icosahedron mesh
    And the resulting Planet has no max depth set

  Scenario: Creating a Planet does not subdivide it
    Given a Planet created with the Earthy preset and seed 1
    Then the resulting Planet's seed is 1
    And the resulting Planet's mesh has 12 vertices
    And the resulting Planet's mesh has the same faces as the icosahedron mesh
    And the resulting Planet's mesh is not identical to the icosahedron mesh
    And the resulting Planet has no max depth set

  Scenario: Subdividing a created Planet produces a new Planet at the requested max depth
    Given a Planet created with the Earthy preset and seed 1
    When that Planet is subdivided to max depth 3
    Then the resulting Planet's max depth is 3
    And the resulting Planet's mesh is identical to a Planet generated with seed 1 and the Earthy preset at max depth 3

  Scenario: A Planet generated with the Earthy preset has approximately its configured ocean quota's fraction of vertices at sea level
    Given a Planet generated with seed 11 and the Earthy preset at max depth 4
    Then the fraction of the resulting Planet's mesh vertices at its minimum vertex radius is within 0.05 of the Earthy preset's configured OceanQuota

  Scenario: A Planet generated with a preset carrying terrace levels has vertex radii clustered at a bounded number of distinct values
    Given a Planet generated with seed 5 and the Volcano preset at max depth 4
    Then the resulting Planet's mesh has at most 6 distinct vertex radii, within floating-point tolerance

  Scenario: An Earthy Planet's post-subdivision vertex radii reflect the increased mountain-height amplitude
    Given a Planet generated with seed 5 and the Earthy preset at max depth 3
    Then every vertex of the resulting Planet's mesh has a radius greater than or equal to 0.4
    And every vertex of the resulting Planet's mesh has a radius less than or equal to 1.6
    And at least one vertex of the resulting Planet's mesh has a radius greater than 1.25

  Scenario: Every vertex at an Earthy Planet's minimum radius renders as its deep-water color, not an elevation-coincidental one
    Given a Planet generated with seed 5 and the Earthy preset at max depth 4
    Then every vertex of the resulting Planet's mesh at its minimum vertex radius has a color equal to the Earthy preset's ColorGradient's first stop's color

  Scenario: A Planet's subdivision mode comes from its preset, not a value independent of preset
    Given a Planet generated with seed 5 and the Earthy preset at max depth 3
    When another Planet is generated with seed 5 and the Volcano preset at max depth 3
    Then both resulting Planets' meshes have exactly 1280 faces
