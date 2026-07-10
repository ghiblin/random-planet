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
    Then every vertex of the resulting Planet's mesh has a radius less than or equal to 1.4
    And every vertex of the resulting Planet's mesh has a radius greater than or equal to 0.05

  Scenario: A Planet generated at zero max depth is exactly the base icosahedron, colored
    Given a Planet generated with seed 1 and the Earthy preset at max depth 0
    Then the resulting Planet's mesh is identical to the icosahedron mesh
    And the resulting Planet has exactly 12 colors

  Scenario: Subdivision depth is honored as a hard cap regardless of the preset's min edge length
    Given a Planet generated with seed 5 and the Volcano preset at max depth 8
    Then the resulting Planet's mesh has no more triangles than 8 rounds of subdivision can produce from an icosahedron

  Scenario: The optional progress callback reports the base mesh and every subdivision round
    Given a recording progress callback
    When a Planet is generated with seed 9 and the Earthy preset at max depth 2 using that callback
    Then the progress callback was invoked 3 times
    And the progress callback's 1st invocation received round 0 with the base icosahedron mesh
    And the progress callback's 3rd invocation received round 2 with the resulting Planet's mesh

  Scenario: The optional progress callback still reports the base mesh at zero max depth
    Given a recording progress callback
    When a Planet is generated with seed 9 and the Earthy preset at max depth 0 using that callback
    Then the progress callback was invoked 1 time
    And the progress callback's 1st invocation received round 0 with the base icosahedron mesh

  Scenario: Building a Planet with no fields set falls back to each field's default
    Given a Planet built with no fields set
    Then the resulting Planet's preset is Earthy
    And the resulting Planet's mesh is identical to a Planet generated with seed 0 and the Earthy preset at max depth 3
